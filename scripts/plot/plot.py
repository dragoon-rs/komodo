# see `benches/README.md`
import json
import sys
import matplotlib.pyplot as plt
import argparse


# # Example
# ```nuon
# [
#     {
#         group: "Alice",
#         items: [
#             [ x, measurement, error ];
#             [ 1, 1143, 120 ],
#             [ 2, 1310, 248 ],
#             [ 4, 1609, 258 ],
#             [ 8, 1953, 343 ],
#             [ 16, 2145, 270 ],
#             [ 32, 3427, 301 ]
#         ]
#     },
#     {
#         group: "Bob",
#         items: [
#             [ x, measurement, error ];
#             [ 1, 2388, 374 ],
#             [ 2, 2738, 355 ],
#             [ 4, 3191, 470 ],
#             [ 8, 3932, 671 ],
#             [ 16, 4571, 334 ],
#             [ 32, 4929, 1094 ]
#         ]
#     },
# ]
# ```
def plot(
    data,
    title: str,
    x_label: str,
    y_label: str,
    save: str = None,
    plot_layout: str = "constrained",
    x_scale: str = "linear",
    y_scale: str = "linear",
):
    fig, ax = plt.subplots(layout=plot_layout)

    for group in data:
        xs = [x["x"] for x in group["items"]]
        ys = [x["measurement"] for x in group["items"]]
        zs = [x["error"] for x in group["items"]]

        down = [y - z for (y, z) in  zip(ys, zs)]
        up = [y + z for (y, z) in zip(ys, zs)]

        style = "dashed" if group["group"].endswith("-ark") else "solid"
        ax.plot(xs, ys, label=group["group"], marker='o', linestyle=style)
        ax.fill_between(xs, down, up, alpha=0.3)

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
    parser = argparse.ArgumentParser()
    parser.add_argument("data", type=str, help="""
        the actual data to show in a multibar plot, here is an example:
        [
            {
                group: "Alice",
                items: [
                    [ x, measurement, error ];
                    [ 1, 1143, 120 ],
                    [ 2, 1310, 248 ],
                    [ 4, 1609, 258 ],
                    [ 8, 1953, 343 ],
                    [ 16, 2145, 270 ],
                    [ 32, 3427, 301 ]
                ]
            },
            {
                group: "Bob",
                items: [
                    [ x, measurement, error ];
                    [ 1, 2388, 374 ],
                    [ 2, 2738, 355 ],
                    [ 4, 3191, 470 ],
                    [ 8, 3932, 671 ],
                    [ 16, 4571, 334 ],
                    [ 32, 4929, 1094 ]
                ]
            },
        ]
                        """)
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
        json.loads(args.data),
        args.title,
        args.x_label,
        args.y_label,
        save=args.save,
        plot_layout=plot_layout,
        x_scale=args.x_scale,
        y_scale=args.y_scale,
    )
