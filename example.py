import geneprog
import numpy as np


def target_function(x, y):
    """This is the goal function that we wish to learn through our genetic
    program. This is kind of cheating, because we know that our function set
    is capable of producing this function. For more of a challenge, we could
    try, e.g., real(y**x)
    """
    z = x**2 + y**2
    return z


if __name__ == "__main__":
    x_min = -1
    x_max = 1
    y_min = -1
    y_max = 1

    n_samples = 500
    xx = np.random.rand(n_samples)*(x_max - x_min) + x_min
    yy = np.random.rand(n_samples)*(y_max - y_min) + y_min

    population_size = 5000
    population = [geneprog.random_tree(3) for _ in range(population_size)]

    for epoch in range(50):
        score = np.zeros(population_size)
        for i, tree in enumerate(population):
            for x, y in zip(xx, yy):
                # let the score be the rms of the error:
                score[i] += (target_function(x, y) - tree.eval(x, y))**2
            # I then add a crude penalty for the complexity of the tree:
            score[i] = (score[i] / xx.shape[0])**0.5 + len(tree.show())*0.001

        print(epoch)
        print(population[np.argsort(score)[0]].show())
        print(np.sort(score)[:10])

        idx = np.random.permutation(
            population_size
        ).reshape([2, population_size//2])
        a_players = score[idx[0]]
        b_players = score[idx[1]]
        winners = np.where(
            a_players < b_players,
            idx[0],
            idx[1],
        )
        winners = [population[i] for i in winners]
        new_population = []  # [population[i] for i in np.argsort(score)[:100]]
        for _ in range(population_size - len(new_population)):
            parent_a = winners[
                np.random.randint(0, high=len(winners))
            ]
            parent_b = winners[
                np.random.randint(0, high=len(winners))
            ]
            new_population.append(geneprog.breed(parent_a, parent_b))
        population = new_population
        for i, tree in enumerate(population):
            if np.random.rand() < 0.2:
                population[i] = geneprog.mutate(tree)
