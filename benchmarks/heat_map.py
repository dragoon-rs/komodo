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


class ParseFloatStrPair(argparse.Action):
    def __init__(self, option_strings, dest, nargs=None, **kwargs):
        super().__init__(option_strings, dest, nargs=nargs, **kwargs)

    def __call__(self, parser, namespace, values, option_string=None):
        pairs = []
        for value in values:
            tokens = value.split(':')
            if len(tokens) != 2:
                parser.error(f"{self.dest}: invalid element '{value}', expected `float:str` pair")
            else:
                tick, label = tokens
                pairs.append((float(tick), label))
        setattr(namespace, self.dest, pairs)


class ParseColorMap(argparse.Action):
    def __init__(self, option_strings, dest, nargs=None, **kwargs):
        super().__init__(option_strings, dest, nargs=nargs, **kwargs)

    def __call__(self, parser, namespace, values, option_string=None):
        if values.startswith("["):
            cmap = mcolors.ListedColormap(np.array(json.loads(values)) / 255)
        else:
            cmap = plt.get_cmap(name=values)

        setattr(namespace, self.dest, cmap)


parser = argparse.ArgumentParser()
parser.add_argument("values"         , nargs="*" , type=float                   )
parser.add_argument("--width" , "-W"             , type=int   , required=True   )
parser.add_argument("--height", "-H"             , type=int   , required=True   )
parser.add_argument("--figsize"      , nargs=2   , type=float                   )
parser.add_argument("--dpi"                      , type=int   , default=300     )
parser.add_argument("--cbar"                     , action="store_true"          )
parser.add_argument("--cbarmin"                  , type=float                   )
parser.add_argument("--cbarmax"                  , type=float                   )
parser.add_argument("--cbarticks"    , nargs="*" , action=ParseFloatStrPair     )
parser.add_argument("--cbarlabel"                , type=str                     )
parser.add_argument("--cbardir"                  , type=str   , choices=["vertical", "horizontal"], default="vertical")
parser.add_argument("--cmap"                     , action=ParseColorMap                           , default="plasma")
parser.add_argument("--clut"                     , type=int                     )
parser.add_argument("--xlabel"                   , type=str                     )
parser.add_argument("--xticks"       , nargs="*" , action=ParseFloatStrPair     )
parser.add_argument("--ylabel"                   , type=str                     )
parser.add_argument("--yticks"       , nargs="*" , action=ParseFloatStrPair     )
parser.add_argument("--title"                    , type=str                     )
parser.add_argument("--save"                     , type=str                     )
parser.add_argument("--overlay"                  , action="store_true"          )
args = parser.parse_args()

if args.width * args.height != len(args.values):
    print(f"bad shape: {len(args.values)} values, -W={args.width}, -H={args.height}")
    exit(1)

if args.figsize is None:
    fig, ax = plt.subplots(layout="constrained")
else:
    fig, ax = plt.subplots(layout="constrained", figsize=args.figsize)

cmap = args.cmap.resampled(args.clut) if args.clut is not None else args.cmap
cmap.set_bad(color="white")

if args.cbarmin is None and args.cbarmax is None:
    norm = mcolors.Normalize()
else:
    if args.cbarmin is None:
        norm = mcolors.Normalize(vmax=args.cbarmax)
    elif args.cbarmax is None:
        norm = mcolors.Normalize(vmin=args.cbarmin)
    else:
        norm = mcolors.Normalize(vmin=args.cbarmin, vmax=args.cbarmax)
mappable = cm.ScalarMappable(norm=norm, cmap=cmap)

if len(args.values) > 0:
    arr = np.array(args.values, dtype=float).reshape((args.height, args.width))
    arr = np.where(arr == None, np.nan, arr)

    if args.cbarmin is None and args.cbarmax is None:
        im = plt.imshow(arr, cmap=cmap)
    else:
        if args.cbarmin is None:
            im = plt.imshow(arr, vmax=args.cbarmax, cmap=cmap)
        elif args.cbarmax is None:
            im = plt.imshow(arr, vmin=args.cbarmin, cmap=cmap)
        else:
            im = plt.imshow(arr, vmin=args.cbarmin, vmax=args.cbarmax, cmap=cmap)

    ax.set_xlabel(args.xlabel)
    ax.set_ylabel(args.ylabel)
    if args.xticks is not None:
        ax.set_xticks(
            [t for t, _ in args.xticks],
            labels=[l for _, l in args.xticks],
        )
    if args.yticks is not None:
        ax.set_yticks(
            [t for t, _ in args.yticks],
            labels=[l for _, l in args.yticks],
        )

    if args.overlay:
        norm = im.norm
        cmap = im.cmap
        rgba = cmap(norm(arr))

        for i in range(arr.shape[0]):
            for j in range(arr.shape[1]):
                # https://stackoverflow.com/a/596243
                r, g, b, _ = rgba[i, j]
                luminance = 0.299*r + 0.587*g + 0.114*b

                ax.text(
                    j, i, arr[i, j],
                    ha="center", va="center",
                    color="black" if luminance > 0.5 else "white",
                )

    if args.title is not None:
        ax.set_title(args.title)

if args.cbar:
    cax = ax if len(args.values) == 0 else None
    cbar = fig.colorbar(mappable, ax=ax, label=args.cbarlabel, orientation=args.cbardir, cax=cax)
    if args.cbarticks is not None:
        cbar.set_ticks([t for t, _ in args.cbarticks])
        cbar.set_ticklabels([l for _, l in args.cbarticks])

if args.save is not None:
    plt.savefig(args.save, dpi=args.dpi)
    print(f"Generated {args.save}")
else:
    plt.show()
