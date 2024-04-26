import json
import sys
import matplotlib.pyplot as plt
import argparse

from typing import List, TypedDict

# all fields of a `Points` should have the same length
class Points(TypedDict):
    x: List[float]
    y: List[float]
    e: List[float]

class LineStyle(TypedDict):
    marker: str
    type: str
    width: int

class Style(TypedDict):
    color: str
    line: LineStyle
    alpha: float

class Graph(TypedDict):
    name: str
    points: Points
    style: Style

HELP = """## Example
```nuon
[
    {
        name: "Alice",
        points: [
            [ x, y, e ];
            [ 1, 1143, 120 ],
            [ 2, 1310, 248 ],
            [ 4, 1609, 258 ],
            [ 8, 1953, 343 ],
            [ 16, 2145, 270 ],
            [ 32, 3427, 301 ]
        ],
        style = {},  # optional, see section below
    },
    {
        name: "Bob",
        points: [
            [ x, y, e ];
            [ 1, 2388, 374 ],
            [ 2, 2738, 355 ],
            [ 4, 3191, 470 ],
            [ 8, 3932, 671 ],
            [ 16, 4571, 334 ],
            [ 32, 4929, 1094 ]
        ]
        style = {},  # optional, see section below
    },
]
```

## Custom style
any record inside the data can have an optional "style" specification.

below is the full shape of that specification, where all of the keys are completely optional,
default values have been chosen:
```nuon
{
    color: null,  # see https://matplotlib.org/stable/users/explain/colors/colors.html
    line: {
        marker: "o",  # see https://matplotlib.org/stable/api/markers_api.html
        type: null,  # see https://matplotlib.org/stable/gallery/lines_bars_and_markers/linestyles.html
        width: null,  # just an integer
    }
    alpha: 0.3,  # a real number between 0 and 1
}
```"""

# see [`HELP`]
def plot(
    graphs: List[Graph],
    title: str,
    x_label: str,
    y_label: str,
    save: str = None,
    plot_layout: str = "constrained",
    x_scale: str = "linear",
    y_scale: str = "linear",
):
    fig, ax = plt.subplots(layout=plot_layout)

    for g in graphs:
        xs = [x["x"] for x in g["points"]]
        ys = [x["y"] for x in g["points"]]
        zs = [x["e"] for x in g["points"]]

        down = [y - z for (y, z) in  zip(ys, zs)]
        up = [y + z for (y, z) in zip(ys, zs)]

        style = {
            "marker": 'o',
            "linestyle": None,
            "color": None,
            "linewidth": None,
        }
        alpha = 0.3
        if "style" in g:
            custom_style = g["style"]
            style["color"] = custom_style.get("color", None)
            style["marker"] = custom_style.get("line", {}).get("marker", style["marker"])
            style["linestyle"] = custom_style.get("line", {}).get("type", style["linestyle"])
            style["linewidth"] = custom_style.get("line", {}).get("width", style["linewidth"])
            alpha = custom_style.get("alpha", alpha)

        ax.plot(xs, ys, label=g["name"], **style)
        if style["color"] is None:
            ax.fill_between(xs, down, up, alpha=alpha)
        else:
            ax.fill_between(xs, down, up, alpha=alpha, color=style["color"])

    ax.set_xlabel(x_label)
    ax.set_ylabel(y_label)

    ax.set_xscale(x_scale)
    ax.set_yscale(y_scale)

    ax.set_title(title)

    ax.legend()
    ax.grid(True)

    if save is not None:
        fig.set_size_inches((16, 9), forward=False)
        fig.savefig(save, dpi=500)

        print(f"plot saved as `{save}`")
    else:
        plt.show()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        formatter_class=argparse.RawTextHelpFormatter
    )
    parser.add_argument("graphs", type=str, help=f"the list of graphs to plot\n\n{HELP}"
)
    parser.add_argument("--title", "-t", type=str, help="the title of the plot")
    parser.add_argument("--x-label", "-x", type=str, help="the x label of the plot")
    parser.add_argument("--y-label", "-y", type=str, help="the y label of the plot")
    parser.add_argument("--x-scale", "-X", type=str, choices=["linear", "log"], default="linear", help="the x scale of the plot")
    parser.add_argument("--y-scale", "-Y", type=str, choices=["linear", "log"], default="linear", help="the y scale of the plot")
    parser.add_argument("--fullscreen", action="store_true")
    parser.add_argument("--save", "-s", type=str, help="a path to save the figure to")
    args = parser.parse_args()

    plot_layout = "constrained" if args.fullscreen else None

    plot(
        json.loads(args.graphs),
        args.title,
        args.x_label,
        args.y_label,
        save=args.save,
        plot_layout=plot_layout,
        x_scale=args.x_scale,
        y_scale=args.y_scale,
    )
