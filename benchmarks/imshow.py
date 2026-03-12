# /// script
# dependencies = [
#     "matplotlib",
#     "pyqt6",
# ]
# ///
import numpy as np

import matplotlib.pyplot as plt
import matplotlib.cm as cm
import matplotlib.colors as mcolors

import argparse
import json


class ParseRGBA2D(argparse.Action):
    def __init__(self, option_strings, dest, nargs=None, **kwargs):
        super().__init__(option_strings, dest, nargs=nargs, **kwargs)

    def __call__(self, parser, namespace, values, option_string=None):
        setattr(namespace, self.dest, np.array(json.loads(values)))

parser = argparse.ArgumentParser()
parser.add_argument("values"                     , action=ParseRGBA2D        )
parser.add_argument("--figsize"      , nargs=2   , type=float                )
parser.add_argument("--dpi"                      , type=int   , default=300  )
parser.add_argument("--save"                     , type=str                  )
args = parser.parse_args()

if args.figsize is None:
    fig, ax = plt.subplots(layout="constrained")
else:
    fig, ax = plt.subplots(layout="constrained", figsize=args.figsize)

plt.imshow(args.values)

ax.set_xticks([])
ax.set_yticks([])

if args.save is not None:
    plt.savefig(args.save, dpi=args.dpi)
    print(f"Generated {args.save}")
else:
    plt.show()
