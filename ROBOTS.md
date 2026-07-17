# Sample robot programs

Short examples for experimenting. Prefer the in-game Programming tips and Robot
programming language guides for the full reference.

## Dump and mine (size ~14)

```
if (move(1.42) < 1.0) {
    rotate(100);
}
while (mine());
dump(2);
dump(3);
```

## OreFinder

Search for high-quality ore (`oreType() == 1`), move toward it, and mine.
Only call `move(oreDistance())` when the distance is greater than zero — if you
are already standing on the ore, distance is 0 and a move does nothing useful.
When mining mixed cells, dump the types you do not want to keep (`dump(2)` /
`dump(3)`, or `dump(0)` for everything) so cargo does not fill with lower-value ore.

```
scan();
bool found = false;
if (oreType() == 1)
{
    if (oreDistance() > 0) {
        move(oreDistance());
    }
    found = true;
    while (mine());
}

scan(60);
if (oreType() == 1)
{
    rotate(60);
    if (oreDistance() > 0) {
        move(oreDistance());
    }
    found = true;
    while (mine());
}

scan(-60);
if (oreType() == 1)
{
    rotate(-60);
    if (oreDistance() > 0) {
        move(oreDistance());
    }
    found = true;
    while (mine());
}

if (!found) {
    while (move(robot.forwardSpeed) < 0.1) {
        rotate(robot.rotateSpeed);
    }
}
```
