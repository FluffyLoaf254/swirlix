I've decided on a new structure to allow for more material information per block:

```
      16               16
-------------------------------
|         BLOCK HEADER        |
|      id       |    length   |
===============================
  1    15       8         8
-------------------------------
|        INTERIOR VOXEL       |
|far|pointer|children| leaves |
===============================
             ....
              32
-------------------------------
|          LEAF VOXEL         |
|         material ref        |
===============================
             ....
              32
-------------------------------
|         FAR POINTER         |
|           pointer           |
===============================
             ....
```

Here,  `children` encodes which division of space the children belong to, `leaves` encodes which of those child nodes are actually leaf nodes. In this structure, you can have up to under 4 billion voxels and each leaf node can have its own material encoded. The materials are still encoded separately so that I can store higher bit-depth information for color and any other attributes can be added in the future.

## Development

This means the simplified buffer is thus:

```
     16         8        8
-------------------------------
|        INTERIOR VOXEL       |
|  pointer  |children| leaves |
===============================
             ....
              32
-------------------------------
|          LEAF VOXEL         |
|         material ref        |
===============================
             ....
```

Which again allows for 65,536 maximum nodes, yet the bit depth on the material provides no restrictions on the amount of materials (much better o(^ o ^)o).
