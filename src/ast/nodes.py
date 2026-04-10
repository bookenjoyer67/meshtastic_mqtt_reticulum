"""
AST Nodes for the language.

This module defines the Abstract Syntax Tree (AST) nodes for the language.
All nodes are implemented as dataclasses for easy construction and pattern matching.
"""

from dataclasses import dataclass, field
from typing import List, Optional, Union, Any
from enum import Enum


class BasicType(Enum):
    """Basic type enumerations for the language."""
    INT = "int"
    FLOAT = "float"
    STRING = "string"
    BOOL = "bool"
    VOID = "void"
    ENERGY = "energy"
    PHASE = "phase"


@dataclass
class Node:
    """Base class for all AST nodes."""
    lineno: int = 0
    col_offset: int = 0


@dataclass
class Program(Node):
    """Root node representing an entire program."""
    statements: List[Node] = field(default_factory=list)


@dataclass
class FunctionDef(Node):
    """Function definition node."""
    name: str
    params: List['Param']
    return_type: Optional['BasicType'] = None
    body: List[Node] = field(default_factory=list)
    annotations: List[Union['EnergyAnnotation', 'PhaseAnnotation']] = field(default_factory=list)


@dataclass
class Param(Node):
    """Function parameter node."""
    name: str
    param_type: Optional['BasicType'] = None
    default_value: Optional['Expr'] = None


@dataclass
class VarDecl(Node):
    """Variable declaration node (mutable)."""
    name: str
    var_type: Optional['BasicType'] = None
    value: Optional['Expr'] = None


@dataclass
class LetDecl(Node):
    """Let declaration node (immutable)."""
    name: str
    var_type: Optional['BasicType'] = None
    value: Optional['Expr'] = None


@dataclass
class AssignStmt(Node):
    """Assignment statement node."""
    target: 'Identifier'
    value: 'Expr'


@dataclass
class IfStmt(Node):
    """If statement node."""
    test: 'Expr'
    body: List[Node] = field(default_factory=list)
    orelse: List[Node] = field(default_factory=list)


@dataclass
class ForStmt(Node):
    """For loop statement node."""
    target: 'Identifier'
    iter: 'Expr'
    body: List[Node] = field(default_factory=list)


@dataclass
class WhileStmt(Node):
    """While loop statement node."""
    test: 'Expr'
    body: List[Node] = field(default_factory=list)


@dataclass
class ReturnStmt(Node):
    """Return statement node."""
    value: Optional['Expr'] = None


@dataclass
class Expr(Node):
    """Base class for all expression nodes."""
    pass


@dataclass
class Call(Expr):
    """Function call expression node."""
    func: 'Identifier'
    args: List[Expr] = field(default_factory=list)


@dataclass
class Identifier(Expr):
    """Identifier expression node."""
    name: str


@dataclass
class Literal(Expr):
    """Literal value expression node."""
    value: Any
    literal_type: Optional[BasicType] = None


@dataclass
class BinaryOp(Expr):
    """Binary operation expression node."""
    left: Expr
    op: str
    right: Expr


@dataclass
class EnergyAnnotation(Node):
    """Energy annotation node."""
    value: Expr


@dataclass
class PhaseAnnotation(Node):
    """Phase annotation node."""
    value: Expr


# Type aliases for cleaner imports
Expr = Union[Call, Identifier, Literal, BinaryOp]
Stmt = Union[VarDecl, LetDecl, AssignStmt, IfStmt, ForStmt, WhileStmt, ReturnStmt]