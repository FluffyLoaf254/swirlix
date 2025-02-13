The buffer data structure will contain three types of u32 values in a contiguous array:

```
      16            1      15
-------------------------------
|         BLOCK HEADER        |
|      id       |unused|length|
===============================
  1     15       8        8
-------------------------------
|            VOXEL            |
|far|pointer|children|material|
===============================
             ....
              32
-------------------------------
|         FAR POINTER         |
|           pointer           |
===============================
             ....
```

The regular, 15-bit voxel pointers are relative to the containing block. The blocks will be laid out as subdivisions of space up to a maximum voxel count of 32,768. The far pointers will be placed at the end of the block, and reachable using the length of the block.

The ID of the block will be used along with the 8-bit material flag to determine the material, providing 256 materials per block.

## Development

To start, I'll begin with this simplified buffer:

```
     16          8        8
-------------------------------
|            VOXEL            |
|  pointer  |children|material|
===============================
            ....
```

This will ease development and make iteration possible. This means there is a maximum of 65,536 voxels total and a maximum of 256 materials/colors.
