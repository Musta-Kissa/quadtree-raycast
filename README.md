![](demo.gif)

# quadtree-raycast
An implementation of the algorithm described in the paper "[An Efficient Parametric Algorithm for Octree Traversal](https://www.researchgate.net/publication/2395157_An_Efficient_Parametric_Algorithm_for_Octree_Traversal)" in rust using minifb for the visualisation
## Running the program
To run it you just have to clone the repo and then run `cargo run --release -- {..args}`
## Options
The program takes several cmd arguments:
|flag |value |desc | 
| --- | ---- | --- |
|`-f` |  -   | initilazes the octree to be full instead of the default empy             |
|`-a` |  -   | visualizes all the nodes hit by the ray instead of the first non empty   |
|`-r` | uint | sets the resolution of the window                                        |
|`-d` | uint | sets the depth of the quadtree                                           | 
### Example
`cargo run --release -- -d 6 -r 720 -f` | this creates a window 720x720 pixels and create a full quadtree with the depth of 6

## Inputs
| button        | action |
| ------------- | ------ |
| `Right Mouse` | Empties the quadtree node at the cursor       |
| `Left Mouse`  | Fills the quatree node at the cursor          |
| `Mouse Pos`   | Determines the origin of the ray              |
| `Arrow Keys`  | Steer the target of the ray (the green circle)|
