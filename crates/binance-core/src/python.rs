//! Python bindings (`python` feature, built with maturin).
//!
//! Two functions are exposed:
//!
//! - `read(config)` where `config` is a TOML path or a dict, returning
//!   `{"dataset": str, "columns": {name: list}, "report": dict}`;
//! - `catalog(root, market=None, period=None, dataset=None, symbol=None)`,
//!   returning the catalog as Python dictionaries.
//!
//! No compatibility layer is provided for the old
//! `load_columns`/`load_kline_columns` API; downstream code should adapt to the
//! column-name based shape.

use std::path::PathBuf;
use std::str::FromStr;

use chrono::NaiveDate;
use pyo3::IntoPyObjectExt;
use pyo3::exceptions::{PyFileNotFoundError, PyOSError, PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use serde::Serialize;
use serde_json::Value;

use crate::{BadRowPolicy, CatalogFilter, Config, Dataset, Interval, Market};

/// Keys accepted by the dict form of [`read`].
const CONFIG_KEYS: [&str; 8] = [
    "root",
    "dataset",
    "market",
    "symbol",
    "interval",
    "start_date",
    "end_date",
    "bad_rows",
];

#[pymodule]
fn binance_core(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(read, module)?)?;
    module.add_function(wrap_pyfunction!(catalog, module)?)?;
    Ok(())
}

/// Read a dataset and return a Python dictionary.
#[pyfunction]
fn read(py: Python<'_>, config: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    let config = parse_config(config)?;
    let data = py.detach(|| crate::read(&config)).map_err(to_pyerr)?;
    let result = PyDict::new(py);
    result.set_item("dataset", config.dataset().as_str())?;
    result.set_item("columns", json_to_py(py, &to_json(&data.table)?)?)?;
    result.set_item("report", json_to_py(py, &to_json(&data.report)?)?)?;
    Ok(result.into_any().unbind())
}

/// Return the archive inventory as a list of dictionaries.
#[pyfunction]
#[pyo3(signature = (root, market=None, period=None, dataset=None, symbol=None))]
fn catalog(
    py: Python<'_>,
    root: &str,
    market: Option<String>,
    period: Option<String>,
    dataset: Option<String>,
    symbol: Option<String>,
) -> PyResult<Py<PyAny>> {
    let filter = CatalogFilter {
        market: parse_filter(market, "market")?,
        period: parse_filter(period, "period")?,
        dataset: parse_filter(dataset, "dataset")?,
        symbol: parse_filter(symbol, "symbol")?,
    };
    let catalog = py
        .detach(|| crate::catalog(std::path::Path::new(root), &filter))
        .map_err(to_pyerr)?;
    json_to_py(py, &to_json(&catalog)?)
}

fn parse_filter<T: FromStr<Err = String>>(raw: Option<String>, field: &str) -> PyResult<Option<T>> {
    raw.map(|value| {
        value
            .parse::<T>()
            .map_err(|e| PyValueError::new_err(format!("{field}: {e}")))
    })
    .transpose()
}

fn parse_config(value: &Bound<'_, PyAny>) -> PyResult<Config> {
    if let Ok(path) = value.extract::<PathBuf>() {
        return Config::from_toml_file(path).map_err(to_pyerr);
    }
    let dict = value.cast::<PyDict>()?;
    reject_unknown_keys(dict, &CONFIG_KEYS)?;

    let root = required_string(dict, "root")?;
    let dataset_raw = required_string(dict, "dataset")?;
    let dataset = dataset_raw
        .parse::<Dataset>()
        .map_err(PyValueError::new_err)?;
    let symbol = required_string(dict, "symbol")?;
    let mut builder = Config::builder(root, dataset, symbol);

    if let Some(market) = optional_string(dict, "market")? {
        builder = builder.market(market.parse::<Market>().map_err(PyValueError::new_err)?);
    }
    if let Some(interval) = optional_string(dict, "interval")? {
        builder = builder.interval(
            interval
                .parse::<Interval>()
                .map_err(PyValueError::new_err)?,
        );
    }
    builder = builder.date_range(
        optional_date(dict, "start_date")?,
        optional_date(dict, "end_date")?,
    );
    if let Some(policy) = optional_string(dict, "bad_rows")? {
        builder = builder.bad_rows(
            policy
                .parse::<BadRowPolicy>()
                .map_err(PyValueError::new_err)?,
        );
    }
    builder.build().map_err(to_pyerr)
}

fn reject_unknown_keys(dict: &Bound<'_, PyDict>, allowed: &[&str]) -> PyResult<()> {
    for key in dict.keys() {
        let key = key
            .extract::<String>()
            .map_err(|_| PyTypeError::new_err("config keys must be strings"))?;
        if !allowed.contains(&key.as_str()) {
            return Err(PyValueError::new_err(format!(
                "unknown config key: {key}; valid keys: {}",
                allowed.join(", ")
            )));
        }
    }
    Ok(())
}

fn optional_string(dict: &Bound<'_, PyDict>, key: &str) -> PyResult<Option<String>> {
    match dict.get_item(key)? {
        Some(value) if !value.is_none() => Ok(Some(value.extract::<String>()?)),
        _ => Ok(None),
    }
}

fn required_string(dict: &Bound<'_, PyDict>, key: &str) -> PyResult<String> {
    optional_string(dict, key)?
        .ok_or_else(|| PyValueError::new_err(format!("missing config key: {key}")))
}

fn optional_date(dict: &Bound<'_, PyDict>, key: &str) -> PyResult<Option<NaiveDate>> {
    optional_string(dict, key)?
        .map(|raw| {
            NaiveDate::parse_from_str(&raw, "%Y-%m-%d")
                .map_err(|e| PyValueError::new_err(format!("{key}: {e}")))
        })
        .transpose()
}

fn to_json<T: Serialize>(value: &T) -> PyResult<Value> {
    serde_json::to_value(value).map_err(|error| to_pyerr(error.into()))
}

fn to_pyerr(error: crate::Error) -> PyErr {
    match &error {
        crate::Error::Config(_) => PyValueError::new_err(error.to_string()),
        crate::Error::RootNotFound { .. } | crate::Error::NoFiles { .. } => {
            PyFileNotFoundError::new_err(error.to_string())
        }
        crate::Error::Io { .. } => PyOSError::new_err(error.to_string()),
        _ => PyRuntimeError::new_err(error.to_string()),
    }
}

fn json_to_py(py: Python<'_>, value: &Value) -> PyResult<Py<PyAny>> {
    match value {
        Value::Null => Ok(py.None()),
        Value::Bool(value) => value.into_py_any(py),
        Value::Number(number) => {
            if let Some(value) = number.as_i64() {
                value.into_py_any(py)
            } else if let Some(value) = number.as_u64() {
                value.into_py_any(py)
            } else if let Some(value) = number.as_f64() {
                value.into_py_any(py)
            } else {
                Err(PyValueError::new_err("JSON number out of range"))
            }
        }
        Value::String(value) => value.as_str().into_py_any(py),
        Value::Array(items) => {
            let list = PyList::empty(py);
            for item in items {
                list.append(json_to_py(py, item)?)?;
            }
            Ok(list.into_any().unbind())
        }
        Value::Object(map) => {
            let dict = PyDict::new(py);
            for (key, item) in map {
                dict.set_item(key.as_str(), json_to_py(py, item)?)?;
            }
            Ok(dict.into_any().unbind())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_config_key_is_a_value_error() {
        Python::initialize();
        Python::attach(|py| {
            let dict = PyDict::new(py);
            dict.set_item("root", "/data").unwrap();
            dict.set_item("dataset", "klines").unwrap();
            dict.set_item("symbol", "BTCUSDT").unwrap();
            dict.set_item("typo", 1).unwrap();

            let error = parse_config(dict.as_any()).unwrap_err();
            assert!(error.is_instance_of::<PyValueError>(py));
            assert!(error.to_string().contains("unknown config key: typo"));
        });
    }

    #[test]
    fn config_error_maps_to_value_error() {
        Python::initialize();
        Python::attach(|py| {
            let error = to_pyerr(crate::Error::Config("bad request".to_owned()));
            assert!(error.is_instance_of::<PyValueError>(py));
        });
    }

    #[test]
    fn missing_root_maps_to_file_not_found() {
        Python::initialize();
        Python::attach(|py| {
            let error = to_pyerr(crate::Error::RootNotFound {
                root: std::path::PathBuf::from("/missing"),
            });
            assert!(error.is_instance_of::<PyFileNotFoundError>(py));
        });
    }

    #[test]
    fn io_error_maps_to_os_error() {
        Python::initialize();
        Python::attach(|py| {
            let error = to_pyerr(crate::Error::io(
                "stat /data",
                std::io::Error::from(std::io::ErrorKind::PermissionDenied),
            ));
            assert!(error.is_instance_of::<PyOSError>(py));
        });
    }
}
