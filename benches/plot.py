import matplotlib.pyplot as plt
import json
import sys
import os
import argparse
from typing import Any, Dict, List

NB_NS_IN_MS = 1e6
NB_BYTES_IN_KB = 1_024

# represents a full NDJSON dataset, i.e. directly generated by `cargo criterion`,
# filtered to remove invalid lines, e.g. whose `$.reason` is not
# `benchmark-complete`
Data = List[Dict[str, Any]]


# k1: namely `mean` or `median`
# k2: namely `estimation`, `upper_bound` or `lower_bound`
def extract_time(data: Data, k1: str, k2: str) -> List[float]:
    return [line[k1][k2] if k2 is not None else line[k1] / NB_NS_IN_MS for line in data]


# read a result dataset from an NDJSON file and filter out invalid lines
#
# here, invalid lines are all the lines with `$.reason` not equal to
# `benchmark-complete` that are generated by `cargo criterion` but useless.
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


def plot_linalg(data: Data):
    # key: the start of the `$.id` field
    def plot(data: Data, key: str, ax):
        filtered_data = list(filter(lambda line: line["id"].startswith(key), data))

        sizes = [
            int(line["id"].split(' ')[1].split('x')[0]) for line in filtered_data
        ]

        means = extract_time(filtered_data, "mean", "estimate")
        up = extract_time(filtered_data, "mean", "upper_bound")
        down = extract_time(filtered_data, "mean", "lower_bound")

        ax.plot(sizes, means, label="mean", color="blue")
        ax.fill_between(sizes, down, up, color="blue", alpha=0.3, label="mean bounds")

        medians = extract_time(filtered_data, "median", "estimate")
        up = extract_time(filtered_data, "median", "upper_bound")
        down = extract_time(filtered_data, "median", "lower_bound")

        ax.plot(sizes, medians, label="median", color="orange")
        ax.fill_between(sizes, down, up, color="orange", alpha=0.3, label="median bounds")

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


def plot_setup(data: Data):
    fig, axs = plt.subplots(4, 1, sharex=True)

    # key: the start of the `$.id` field
    # i: the index where the size of the input data needs to be extracted from
    def plot(data: Data, key: str, i: int, label: str, color: str, error_bar: bool, ax):
        filtered_data = list(filter(lambda line: line["id"].startswith(key), data))
        sizes = [int(line["id"].split(' ')[i]) / NB_BYTES_IN_KB for line in filtered_data]

        if error_bar:
            means = extract_time(filtered_data, "mean", "estimate")
            up = extract_time(filtered_data, "mean", "upper_bound")
            down = extract_time(filtered_data, "mean", "lower_bound")
        else:
            means = [x * NB_NS_IN_MS / NB_BYTES_IN_KB for x in extract_time(filtered_data, "mean", None)]

        ax.plot(sizes, means, label=label, color=color)

        if error_bar:
            ax.fill_between(sizes, down, up, color=color, alpha=0.3, label="mean bounds")

    # setup
    plot(data, "setup/setup", 1, "mean", "orange", True, axs[0])
    axs[0].set_title("time to generate a random trusted setup")
    axs[0].set_ylabel("time (in ms)")
    axs[0].legend()
    axs[0].grid()

    # serialization
    plot(data, "setup/serializing with compression", -3, "mean compressed", "orange", True, axs[1])
    plot(data, "setup/serializing with no compression", -3, "mean uncompressed", "blue", True, axs[1])
    axs[1].set_title("serialization")
    axs[1].set_ylabel("time (in ms)")
    axs[1].legend()
    axs[1].grid()

    # deserialization
    plot(data, "setup/deserializing with no compression and no validation", -3, "mean uncompressed unvalidated", "red", True, axs[2])
    plot(data, "setup/deserializing with compression and no validation", -3, "mean compressed unvalidated", "orange", True, axs[2])
    plot(data, "setup/deserializing with no compression and validation", -3, "mean uncompressed validated", "blue", True, axs[2])
    plot(data, "setup/deserializing with compression and validation", -3, "mean compressed validated", "green", True, axs[2])
    axs[2].set_title("deserialization")
    axs[2].set_ylabel("time (in ms)")
    axs[2].legend()
    axs[2].grid()

    plot(data, "serialized size with no compression and no validation", -3, "mean uncompressed unvalidated", "red", False, axs[3])
    plot(data, "serialized size with compression and no validation", -3, "mean compressed unvalidated", "orange", False, axs[3])
    plot(data, "serialized size with no compression and validation", -3, "mean uncompressed validated", "blue", False, axs[3])
    plot(data, "serialized size with compression and validation", -3, "mean compressed validated", "green", False, axs[3])
    axs[3].set_title("size")
    axs[3].set_xlabel("number of expected bytes (in kb)")
    axs[3].set_ylabel("size (in kb)")
    axs[3].legend()
    axs[3].grid()

    plt.show()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("filename", type=str)
    parser.add_argument(
        "--bench", "-b", type=str, choices=["linalg", "setup"], required=True
    )
    args = parser.parse_args()

    data = read_data(args.filename)

    match args.bench:
        case "linalg":
            plot_linalg(data)
        case "setup":
            plot_setup(data)