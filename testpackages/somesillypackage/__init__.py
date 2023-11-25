import datetime
from os import path

import somesillypackage.a
import somesillypackage.child1.a

def foo():
    from somesillypackage import b
    from somesillypackage.child1 import b

class Bar:
    from somesillypackage.c import C
    from somesillypackage.child1.c import C

from . import d
from .child1 import d

from .e import E
from .child1.e import E

import somesillypackage.child1
from somesillypackage import child2
from somesillypackage.child3 import CHILD
from . import child4
from .child5 import CHILD