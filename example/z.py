import datetime
from os import path

import example.a
import example.child.c_a

from example import b
from example.child import c_b

from example.c import C
from example.child.c_c import C

from . import d
from .child import c_d

from .e import E
from .child.c_e import E

import example.child
from example import child2
from example.child3 import CHILD
from . import child4
from .child5 import CHILD