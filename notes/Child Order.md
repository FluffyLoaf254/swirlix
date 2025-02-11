The child order will be this:

- 1st Bit: Left Front Bottom (LFB) / -x -y -z
- 2nd Bit: Right Front Bottom (RFB) / +x -y -z
- 3rd Bit: Left Back Bottom (LBB) / -x +y -z
- 4th Bit: Right Back Bottom (RBB) / +x +y -z
- 5th Bit: Left Front Top (LFT) / -x -y +z
- 6th Bit: Right Front Top (RFT) / +x -y +z
- 7th Bit: Left Back Top (LBT) / -x +y +z
- 8th Bit: Right Back Top (RBT) / +x +y +z

That is, it means this:

```
child_mask & (255 >> 7) == 1st / LFB
(child_mask >> 1) & (255 >> 6) == 2nd / RFB
(child_mask >> 2) & (255 >> 5) == 3rd / LBB
(child_mask >> 3) & (255 >> 4) == 4th / RBB
(child_mask >> 4) & (255 >> 3) == 5th / LFT
(child_mask >> 5) & (255 >> 2) == 6th / RFT
(child_mask >> 6) & (255 >> 1) == 7th / LBT
child_mask >> 7 == 8th / RBT
```
