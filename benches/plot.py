import matplotlib.pyplot as plt
import json
import sys
import os
import argparse
from typing import Any, Dict, List

NB_NS_IN_MS = 1e6
NB_BYTES_IN_KB = 1_024

FULLSCREEN_DPI = 300

# represents a full NDJSON dataset, i.e. directly generated by `cargo criterion`,
# filtered to remove invalid lines, e.g. whose `$.reason` is not
# `benchmark-complete`
Data = List[Dict[str, Any]]


# k1: namely `mean` or `median`
# k2: namely `estimation`, `upper_bound`, `lower_bound` or None
def extract(data: Data, k1: str, k2: str) -> List[float]:
    return [line[k1][k2] if k2 is not None else line[k1] for line in data]


# convert a list of times in nanoseconds to the same list in milliseconds
def ns_to_ms(times: List[float]) -> List[float]:
    return [t / NB_NS_IN_MS for t in times]


# convert a list of sizes in bytes to the same list in kilobytes
def b_to_kb(sizes: List[int]) -> List[float]:
    return [s / NB_BYTES_IN_KB for s in sizes]


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


def plot_linalg(data: Data, save: bool = False):
    # key: the start of the `$.id` field
    def plot(data: Data, key: str, ax):
        filtered_data = list(filter(lambda line: line["id"].startswith(key), data))

        sizes = [
            int(line["id"].split(' ')[1].split('x')[0]) for line in filtered_data
        ]

        means = ns_to_ms(extract(filtered_data, "mean", "estimate"))
        up = ns_to_ms(extract(filtered_data, "mean", "upper_bound"))
        down = ns_to_ms(extract(filtered_data, "mean", "lower_bound"))

        ax.plot(sizes, means, label="mean", color="blue")
        ax.fill_between(sizes, down, up, color="blue", alpha=0.3)

        medians = ns_to_ms(extract(filtered_data, "median", "estimate"))
        up = ns_to_ms(extract(filtered_data, "median", "upper_bound"))
        down = ns_to_ms(extract(filtered_data, "median", "lower_bound"))

        ax.plot(sizes, medians, label="median", color="orange")
        ax.fill_between(sizes, down, up, color="orange", alpha=0.3)

    labels = ["transpose", "mul", "inverse"]

    fig, axs = plt.subplots(len(labels), 1, figsize=(16, 9))

    for label, ax in zip(labels, axs):
        plot(data, key=label, ax=ax)
        ax.set_title(label)
        ax.set_yscale("log")
        ax.set_ylabel("time in ms")
        ax.legend()
        ax.grid()

    if save:
        output = "linalg.png"
        plt.savefig(output, dpi=FULLSCREEN_DPI)
        print(f"figure saved as {output}")
    else:
        plt.show()


def plot_setup(data: Data, save: bool = False):
    fig, axs = plt.subplots(4, 1, sharex=True, figsize=(16, 9))

    # key: the start of the `$.id` field
    def plot(data: Data, key: str, label: str, color: str, error_bar: bool, ax):
        filtered_data = list(filter(lambda line: line["id"].startswith(key), data))
        sizes = [int(line["id"].lstrip(key).split(' ')[0]) for line in filtered_data]

        if error_bar:
            means = ns_to_ms(extract(filtered_data, "mean", "estimate"))
            up = ns_to_ms(extract(filtered_data, "mean", "upper_bound"))
            down = ns_to_ms(extract(filtered_data, "mean", "lower_bound"))
        else:
            means = b_to_kb(extract(filtered_data, "mean", None))

        ax.plot(sizes, means, label=label, color=color)

        if error_bar:
            ax.fill_between(sizes, down, up, color=color, alpha=0.3)

    # setup
    plot(data, "setup/setup (komodo)", "komodo", "orange", True, axs[0])
    plot(data, "setup (arkworks)", "arkworks", "blue", True, axs[0])
    axs[0].set_title("time to generate a random trusted setup")
    axs[0].set_ylabel("time (in ms)")
    axs[0].legend()
    axs[0].grid()

    # serialization
    plot(data, "setup/serializing with compression", "compressed", "orange", True, axs[1])
    plot(data, "setup/serializing with no compression", "uncompressed", "blue", True, axs[1])
    axs[1].set_title("serialization")
    axs[1].set_ylabel("time (in ms)")
    axs[1].legend()
    axs[1].grid()

    # deserialization
    plot(data, "setup/deserializing with no compression and no validation", "uncompressed unvalidated", "red", True, axs[2])
    plot(data, "setup/deserializing with compression and no validation", "compressed unvalidated", "orange", True, axs[2])
    plot(data, "setup/deserializing with no compression and validation", "uncompressed validated", "blue", True, axs[2])
    plot(data, "setup/deserializing with compression and validation", "compressed validated", "green", True, axs[2])
    axs[2].set_title("deserialization")
    axs[2].set_ylabel("time (in ms)")
    axs[2].legend()
    axs[2].grid()

    plot(data, "serialized size with no compression and no validation", "uncompressed unvalidated", "red", False, axs[3])
    plot(data, "serialized size with compression and no validation", "compressed unvalidated", "orange", False, axs[3])
    plot(data, "serialized size with no compression and validation", "uncompressed validated", "blue", False, axs[3])
    plot(data, "serialized size with compression and validation", "compressed validated", "green", False, axs[3])
    axs[3].set_title("size")
    axs[3].set_xlabel("degree")
    axs[3].set_ylabel("size (in kb)")
    axs[3].legend()
    axs[3].grid()

    if save:
        output = "setup.png"
        plt.savefig(output, dpi=FULLSCREEN_DPI)
        print(f"figure saved as {output}")
    else:
        plt.show()


def plot_commit(data: Data, save: bool = False):
    fig, ax = plt.subplots(1, 1, figsize=(16, 9))

    # key: the start of the `$.id` field
    def plot(data: Data, key: str, color: str, ax):
        filtered_data = list(filter(lambda line: line["id"].startswith(key), data))

        sizes = [
            int(line["id"].lstrip(key).split(' ')[0]) for line in filtered_data
        ]

        means = ns_to_ms(extract(filtered_data, "mean", "estimate"))
        up = ns_to_ms(extract(filtered_data, "mean", "upper_bound"))
        down = ns_to_ms(extract(filtered_data, "mean", "lower_bound"))

        ax.plot(sizes, means, label=key, color=color)
        ax.fill_between(sizes, down, up, color=color, alpha=0.3)

    keys = ["commit (komodo)", "commit (arkworks)"]
    colors = ["blue", "orange"]

    for (k, c) in zip(keys, colors):
        plot(data, key=k, color=c, ax=ax)

    ax.set_title("commit times")
    ax.set_ylabel("time (in ms)")
    ax.set_xlabel("degree")
    ax.legend()
    ax.grid(True)

    if save:
        output = "commit.png"
        plt.savefig(output, dpi=FULLSCREEN_DPI)
        print(f"figure saved as {output}")
    else:
        plt.show()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("filename", type=str)
    parser.add_argument(
        "--bench", "-b", type=str, choices=["linalg", "setup", "commit"],
    )
    parser.add_argument(
        "--save", "-s", action="store_true", default=False,
    )
    parser.add_argument(
        "--all", "-a", action="store_true", default=False,
    )
    args = parser.parse_args()

    data = read_data(args.filename)

    if args.all:
        plot_linalg(data, save=args.save)
        plot_setup(data, save=args.save)
        plot_commit(data, save=args.save)
        exit(0)

    match args.bench:
        case "linalg":
            plot_linalg(data, save=args.save)
        case "setup":
            plot_setup(data, save=args.save)
        case "commit":
            plot_commit(data, save=args.save)
        case _:
            print("nothing to do: you might want to use `--bench <bench>` or `--all`")
