The abstract for the ant algorithm is as follows.

Let A be the set of all information an ant has on a given turn. A = {h, f, H0, H1, H2, H3, H4, H5, H6}
    Let h be an ordered pair of integers indicating the coordinates of the nest on the map.
    Let f be a Boolean. f is true if and only if the ant is carrying food on this turn. An ant can only carry 1 unit of food.
    For 0 <= n <= 6, let Hn be hexes the ant is occupying or adjacent to. H0 is the occupied hex. Other hexes are adjacent, starting with the right hex going counterclockwise. Hn = {fn, pn, qn}
        Let fn be an unsigned integer for the amount of food occupying the hex. fn is decremented by 1 whenever an ant takes food from the hex.
        Let pn be a float for a pheromone indicating a path from food to the nest. pn has exponential decay.
        Let qn be a float for a pheromone indicating a hex an ant has already searched. qn has exponential decay.

Let B be the set of all move actions an ant can take on a given turn. An ant must make exactly one move action per turn. B = {c, m1, m2, m3, m4, m5, m6}
    Let c be the grab/release action for food. An ant can only grab food if it is on food and has no food and can only release food if it is on the nest and has food.
    For 1 <= n <= 6, let mn be a move to an adjacent hex, starting with the right hex going counterclockwise.

Let C be the set of all pheromones an ant can release on a given turn. An ant always releases a given pheromone at a rate of 1 unit per turn on the occupied hex. C = {p, q}
    Let p be a pheromone indicating a path from food to the nest. p is released if and only if f is true.
    Let q be a pheromone indicating a hex an ant has already searched. q is always being released.

Let A* be a set containing all possible instances of A.

*If there is a tie at any point, then choose a tiebreaker at random with uniform probability distribution.*
Let F: A* -> B be a function that chooses a move action in B given all of the information in A.
    If f is true:
        If the ant is on the nest, then c.
        Else, move to the Hn in A with the greatest pn that puts the ant closer to the nest.
    Else:
        If f0 in H0 is nonzero, then c.
        Else, if there exists an Hn in A with a nonzero fn, n != 0, then move to the Hn with the greatest fn.
        Else, if there exists an Hn in A with a nonzero pn, and moving to Hn would put the ant at a greater distance from the nest, then move to the Hn with the greatest pn that would put the ant farther from the nest.
        Else, move to the Hn in A with the least qn.
