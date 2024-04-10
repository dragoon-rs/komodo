# ## usage
# ```nushell
# let res = cargo run --example bench_commit
#     | lines
#     | parse "{curve}: {degree} -> {t}"
#     | into int degree
#     | update t { into int | into duration }
#
# python scripts/plot/bench_commit.py (
#     $res | group-by curve --to-table | update items { reject curve } | to json
# )
# ```
import json
import sys
import matplotlib.pyplot as plt

NB_NS_IN_MS = 1e6


if __name__ == "__main__":
    data = json.loads(sys.argv[1])

    for group in data:
        xs = [x["degree"] for x in group["items"]]
        ys = [x["t"] / NB_NS_IN_MS for x in group["items"]]

        plt.plot(xs, ys, label=group["group"], marker='o')

    plt.xlabel("degree")
    plt.ylabel("time (in ns)")

    plt.title("time to commit polynomials for certain curves")

    plt.legend()
    plt.grid(True)
    plt.show()
