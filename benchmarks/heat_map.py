# /// script
# dependencies = [
#     "matplotlib",
#     "pyqt6",
# ]
# ///
import numpy as np
import matplotlib.pyplot as plt

import argparse


parser = argparse.ArgumentParser()
parser.add_argument("values"         , nargs="+" , type=float                 )
parser.add_argument("--width" , "-W"             , type=int   , required=True )
parser.add_argument("--height", "-H"             , type=int   , required=True )
parser.add_argument("--figsize"      , nargs=2   , type=int   , default=(16,9))
parser.add_argument("--dpi"                      , type=int   , default=300   )
parser.add_argument("--save"                     , type=str                   )
args = parser.parse_args()

if args.width * args.height != len(args.values):
    print(f"bad shape: {len(args.values)} values, -W={args.width}, -H={args.height}")
    exit(1)

arr = np.array(args.values, dtype=float).reshape((args.height, args.width))
arr = np.where(arr == None, np.nan, arr)

fig, ax = plt.subplots(layout="constrained", figsize=args.figsize)

cmap = plt.cm.plasma.copy()
cmap.set_bad(color="white")

plt.imshow(arr, cmap=cmap)
plt.colorbar()

if args.save is not None:
    plt.savefig(args.save, dpi=args.dpi)
    print(f"Generated {args.save}")
else:
    plt.show()
