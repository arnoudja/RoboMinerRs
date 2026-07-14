Dump and mine, size 14
if (move(1.42) < 1.0) {
    rotate(100);
}
while(mine());
dump(2);
dump(3);


OreFinder, size 55
scan();
bool found = false;
if (oreType() == 1)
{
    move(oreDistance());
    found = true;
    while (mine());
}

scan(60);
if (oreType() == 1)
{
    rotate(60);
    move(oreDistance());
    found = true;
    while (mine());
}

scan(-60);
if (oreType() == 1)
{
    rotate(-60);
    move(oreDistance());
    found = true;
    while (mine());
}

if (!found) {
    while (move(robot.forwardSpeed) < 0.1) {
        rotate(robot.rotateSpeed);
    }
}
