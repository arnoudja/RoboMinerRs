
delete from PoolItemMiningTotals;
delete from PoolItem;
delete from Pool;


insert into Pool
(id, miningAreaId, requiredRuns)
values
(1, 1401, 2500);

insert into PoolItem
(poolId, robotId, sourceCode)
values
(1, 6, '
int rot = 0;

rotate(5);

while (move(4.25) > 3.9 && !mine());

while (true)
{
    while (mine())
    {
        rot = 100;
    }
    if (rot > 0)
    {
        if (rot < 100)
        {
            rotate(rot);
        }
        rot = rot - 10;
    }
    if (move(1.415) < 1.4)
    {
        move(-1.415);
        rotate(45);
    }
}
');

insert into PoolItem
(poolId, robotId, sourceCode)
values
(1, 6, '
int rot = 0;

rotate(5);

while (true)
{
    while (mine())
    {
        rot = 100;
    }
    if (rot > 0)
    {
        if (rot < 100)
        {
            rotate(rot);
        }
        rot = rot - 10;
    }
    if (move(1.415) < 1.4)
    {
        move(-1.415);
        rotate(22);
    }
}
');

insert into PoolItem
(poolId, robotId, sourceCode)
values
(1, 6, '
//while (!mine() && (move(2)>0.1))
while (!mine())
{
    move(2);
}
bool clockwise = false;
bool hasmined = false;

while(true)
{
  int emptysteps = 0;
  while (emptysteps <1)
  {
    while (mine())
    {
      emptysteps = 0;
      hasmined = true;
    }
    if(move(1.42) < 0.1)
    {
      rotate(45*clockwise);
    }
    emptysteps++;
  }

  if((mine() < 1) && hasmined)
  { 
    int rotation = 90-180*clockwise;
    rotate(rotation);
    move(0.71);
    rotate(rotation);
    clockwise = !clockwise;
    hasmined = false;
  }
}
');

insert into PoolItem
(poolId, robotId, sourceCode)
values
(1, 6, '
while (!mine())
{
  move(2);
}

bool hasmined = false;
bool clockwise = false;

while(true)
{
  int emptysteps = 0;
  while (emptysteps <2)
  {
    while (mine())
    {
      emptysteps = 0;
      hasmined = true;
    }
    move(1.5);
    emptysteps++;
  }

  if((mine() < 1) && hasmined)
  { 
    rotate(135*clockwise);
    hasmined = false;
    clockwise=!clockwise
  }
}
');
