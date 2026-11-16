"""Type stubs for the ``dukascopy_core`` extension module."""

from typing import Any, Optional, TypedDict

import numpy as np
from numpy.typing import NDArray

class _ConfigRequired(TypedDict):
    root: str
    symbol: str
    timeframe: str

class Config(_ConfigRequired, total=False):
    price_type: str
    start_date: str
    end_date: str

class Columns(TypedDict):
    time: NDArray[np.int64]
    open: NDArray[np.float64]
    high: NDArray[np.float64]
    low: NDArray[np.float64]
    close: NDArray[np.float64]
    volume: NDArray[np.uint32]

class Report(TypedDict):
    requested_files: int
    loaded_files: int
    missing_files: list[str]
    rows_decoded: int
    rows_returned: int
    rows_trimmed: int
    rows_invalid: int
    rows_out_of_bucket: int
    duplicates_removed: int
    first_bar_ms: Optional[int]
    last_bar_ms: Optional[int]
    complete: bool
    gaps: dict[str, int]

def load_columns(config: Config) -> Columns: ...
def load(config: Config) -> tuple[Columns, Report]: ...
