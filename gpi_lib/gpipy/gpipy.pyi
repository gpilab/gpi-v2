"""Interface to GPI"""
# This file contains all the type/documentation information for the gpipy module
# It is manually created, because `pyo3` can't automatically generate it yet

from typing import Dict

class GpiNode:
    """doc string for GpiNode!"""
    def __init__(self, a: int, b: int, out: int, config: Dict) -> None: ...

class Gadget:
    """doc string for gadget!"""

    prop: int
    def __init__(self) -> None: ...
    def push(self, v: int) -> None: ...