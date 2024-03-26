import matplotlib.pyplot as plt
import json
import sys
import os
from typing import Any, Dict, List

NB_NS_IN_MS = 1e6

Data = List[Dict[str, Any]]


def extract(data: Data, k1: str, k2: str) -> List[float]:
    return [line[k1][k2] / NB_NS_IN_MS for line in data]


def plot(data: Data, key: str, ax):
    filtered_data = list(filter(lambda line: line["id"].startswith(key), data))

    sizes = [
        int(line["id"].split(' ')[1].split('x')[0]) for line in filtered_data
    ]

    means = extract(filtered_data, "mean", "estimate")
    up = extract(filtered_data, "mean", "upper_bound")
    down = extract(filtered_data, "mean", "lower_bound")

    ax.plot(sizes, means, label="mean", color="blue")
    ax.fill_between(sizes, down, up, color="blue", alpha=0.3, label="mean bounds")

    medians = extract(filtered_data, "median", "estimate")
    up = extract(filtered_data, "median", "upper_bound")
    down = extract(filtered_data, "median", "lower_bound")

    ax.plot(sizes, medians, label="median", color="orange")
    ax.fill_between(sizes, down, up, color="orange", alpha=0.3, label="median bounds")


def parse_args():
    if len(sys.argv) == 1:
        print("please give a filename as first positional argument")
        exit(1)

    return sys.argv[1]


def read_data(data_file: str) -> Data:
    if not os.path.exists(data_file):
        print(f"no such file: `{data_file}`")
        exit(1)

    with open(data_file, "r") as file:
        data = list(filter(
            lambda line: line["reason"] == "benchmark-complete",
            map(
                json.loads,
                file.readlines()
            )
        ))

    return data


if __name__ == "__main__":
    results_file = parse_args()
    data = read_data(results_file)

    labels = ["transpose", "mul", "inverse"]

    fig, axs = plt.subplots(len(labels), 1)

    for label, ax in zip(labels, axs):
        plot(data, key=label, ax=ax)
        ax.set_title(label)
        ax.set_yscale("log")
        ax.set_ylabel("time in ms")
        ax.legend()
        ax.grid()

    plt.show()
