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
def plot(data, title: str, x_label: str, y_label: str, save: str = None):
    for group in data:
        xs = [x["x"] for x in group["items"]]
        ys = [x["measurement"] for x in group["items"]]
        zs = [x["error"] for x in group["items"]]

        down = [y - z for (y, z) in  zip(ys, zs)]
        up = [y + z for (y, z) in zip(ys, zs)]

        style = "dashed" if group["group"].endswith("-ark") else "solid"
        plt.plot(xs, ys, label=group["group"], marker='o', linestyle=style)
        plt.fill_between(xs, down, up, alpha=0.3)

    plt.xlabel(x_label)
    plt.ylabel(y_label)

    plt.title(title)

    plt.legend()
    plt.grid(True)

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
    parser.add_argument("--save", "-s", type=str, help="a path to save the figure to")
    args = parser.parse_args()

    plot(json.loads(args.data), args.title, args.x_label, args.y_label, save=args.save)
