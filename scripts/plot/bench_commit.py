# see `benches/README.md`
import json
import sys
import matplotlib.pyplot as plt

NB_NS_IN_MS = 1e6


if __name__ == "__main__":
    data = json.loads(sys.argv[1])

    for group in data:
        xs = [x["degree"] for x in group["items"]]
        ys = [x["mean"] / NB_NS_IN_MS for x in group["items"]]
        zs = [x["stddev"] / NB_NS_IN_MS for x in group["items"]]

        down = [y - z for (y, z) in  zip(ys, zs)]
        up = [y + z for (y, z) in zip(ys, zs)]

        style = "dashed" if group["group"].endswith("-ark") else "solid"
        plt.plot(xs, ys, label=group["group"], marker='o', linestyle=style)
        plt.fill_between(xs, down, up, alpha=0.3)

    plt.xlabel("degree")
    plt.ylabel("time (in ns)")

    plt.title("time to commit polynomials for certain curves")

    plt.legend()
    plt.grid(True)
    plt.show()
