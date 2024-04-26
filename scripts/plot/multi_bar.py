from typing import Dict, List

import matplotlib.pyplot as plt
import numpy as np
import json
import sys
import argparse


# convert raw data into groups and measurements
#
# # Example
# input:
# ```json
# {
#     "age": {
#         "alice": 31,
#         "bob": 28,
#         "charlie": 44
#     },
#     "height": {
#         "alice": 1.50,
#         "bob": 1.82,
#         "charlie": 1.65
#     },
#     "weight": {
#         "alice": 65.3,
#         "bob": 98.1,
#         "charlie": 68.7
#     }
# }
# ```
#
# output:
# ```
# groups = ["age", "height", "weight"]
# measurements = {
#      "alice": [1.50, 31, 65.3],
#      "bob": [1.82, 28, 98.1],
#      "charlie": [1.65, 44, 68.7]
# }
# ```
def extract(data: Dict[str, Dict[str, float]]) -> (List[str], Dict[str, List[float]]):
    groups = list(data.keys())

    measurements = {}
    for x in data[groups[0]].keys():
        measurements[x] = [data[g][x] for g in groups]

    return (groups, measurements)


# plot multi bars
#
# # Example
# input can be the output of [`extract`]
def plot_multi_bar(
    groups: List[str],
    measurements: Dict[str, List[float]],
    title: str,
    y_label: str,
    labels_locations: List[float] = None,
    width: float = 0.25,
    nb_legend_cols: int = 3,
    legend_loc: str = "upper left",
    plot_layout: str = "constrained",
    save: str = None,
):
    if labels_locations is None:
        labels_locations = np.arange(len(groups))

    fig, ax = plt.subplots(layout=plot_layout)

    multiplier = 0
    for attribute, measurement in measurements.items():
        offset = width * multiplier
        rects = ax.bar(labels_locations + offset, measurement, width, label=attribute)
        ax.bar_label(rects, padding=3)
        multiplier += 1

    ax.set_ylabel(y_label)
    ax.set_title(title)
    ax.set_xticks(labels_locations + width, groups)
    ax.legend(loc=legend_loc, ncols=nb_legend_cols)

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
        {
            "age": {
                "alice": 31,
                "bob": 28,
                "charlie": 44
            },
            "height": {
                "alice": 1.50,
                "bob": 1.82,
                "charlie": 1.65
            },
            "weight": {
                "alice": 65.3,
                "bob": 98.1,
                "charlie": 68.7
            }
        }
                        """)
    parser.add_argument("--title", "-t", type=str, help="the title of the multibar plot")
    parser.add_argument("--label", "-l", type=str, help="the measurement label of the multibar plot")
    parser.add_argument("--fullscreen", action="store_true")
    parser.add_argument("--save", "-s", type=str, help="a path to save the figure to")
    args = parser.parse_args()

    groups, measurements = extract(json.loads(args.data))

    plot_layout = "constrained" if args.fullscreen else None

    plot_multi_bar(
        groups, measurements, args.title, args.label, save=args.save, plot_layout=plot_layout
    )
