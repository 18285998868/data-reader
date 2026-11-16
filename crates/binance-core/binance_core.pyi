"""Type stubs for the ``binance_core`` extension module."""

from os import PathLike
from typing import Any, Optional, TypedDict, Union

class _ConfigRequired(TypedDict):
    root: str
    dataset: str
    symbol: str

class Config(_ConfigRequired, total=False):
    market: str
    interval: str
    start_date: str
    end_date: str
    bad_rows: str

class ReadResult(TypedDict):
    dataset: str
    columns: dict[str, Any]
    report: dict[str, Any]

def read(config: Union[str, PathLike[str], Config]) -> ReadResult: ...
def catalog(
    root: str,
    market: Optional[str] = ...,
    period: Optional[str] = ...,
    dataset: Optional[str] = ...,
    symbol: Optional[str] = ...,
) -> list[dict[str, Any]]: ...
