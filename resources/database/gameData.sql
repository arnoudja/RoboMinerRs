SET storage_engine=InnoDB;

-- Cleaning unreferenced tables always completely filled with the data in this file.
delete from OrePriceAmount;
delete from MiningAreaOreSupply;
delete from AchievementStepMiningTotalRequirement;
delete from AchievementStepMiningScoreRequirement;
delete from AchievementStep;
delete from AchievementPredecessor;


-- The ore type names
insert into Ore (id, oreName) values (1, 'Cerbonium') ON DUPLICATE KEY UPDATE oreName = VALUES(oreName);
insert into Ore (id, oreName) values (2, 'Oxaria') ON DUPLICATE KEY UPDATE oreName = VALUES(oreName);
insert into Ore (id, oreName) values (3, 'Lithabine') ON DUPLICATE KEY UPDATE oreName = VALUES(oreName);
insert into Ore (id, oreName) values (4, 'Neudralion') ON DUPLICATE KEY UPDATE oreName = VALUES(oreName);
insert into Ore (id, oreName) values (5, 'Complatix') ON DUPLICATE KEY UPDATE oreName = VALUES(oreName);
insert into Ore (id, oreName) values (6, 'Prantum') ON DUPLICATE KEY UPDATE oreName = VALUES(oreName);
insert into Ore (id, oreName) values (7, 'Raxia') ON DUPLICATE KEY UPDATE oreName = VALUES(oreName);
insert into Ore (id, oreName) values (8, 'Dipolir') ON DUPLICATE KEY UPDATE oreName = VALUES(oreName);
insert into Ore (id, oreName) values (9, 'Asradon') ON DUPLICATE KEY UPDATE oreName = VALUES(oreName);
insert into Ore (id, oreName) values (10, 'Baratiem') ON DUPLICATE KEY UPDATE oreName = VALUES(oreName);
insert into Ore (id, oreName) values (11, 'Etaxy') ON DUPLICATE KEY UPDATE oreName = VALUES(oreName);

-- The robot part names
insert into RobotPartType (id, typeName) values (1, 'Ore container') ON DUPLICATE KEY UPDATE typeName = VALUES(typeName);
insert into RobotPartType (id, typeName) values (2, 'Mining unit') ON DUPLICATE KEY UPDATE typeName = VALUES(typeName);
insert into RobotPartType (id, typeName) values (3, 'Battery') ON DUPLICATE KEY UPDATE typeName = VALUES(typeName);
insert into RobotPartType (id, typeName) values (4, 'Memory module') ON DUPLICATE KEY UPDATE typeName = VALUES(typeName);
insert into RobotPartType (id, typeName) values (5, 'CPU') ON DUPLICATE KEY UPDATE typeName = VALUES(typeName);
insert into RobotPartType (id, typeName) values (6, 'Engine') ON DUPLICATE KEY UPDATE typeName = VALUES(typeName);
insert into RobotPartType (id, typeName) values (7, 'Ore scanner') ON DUPLICATE KEY UPDATE typeName = VALUES(typeName);

-- Shop prices - Cerbonium
insert into OrePrice (id, description) values (101, 'Standard price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount) values (101, 1, 2);

insert into OrePrice (id, description) values (102, 'Enhanced price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount) values (102, 1, 5);

insert into OrePrice (id, description) values (103, 'Cerbonium price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount) values (103, 1, 15);

-- Shop prices - Oxaria
insert into OrePrice (id, description) values (201, 'Oxaria price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount) values (201, 2, 5);
insert into OrePriceAmount (orePriceId, oreId, amount) values (201, 1, 20);

insert into OrePrice (id, description) values (202, 'Enhanced Oxaria price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount) values (202, 2, 15);
insert into OrePriceAmount (orePriceId, oreId, amount) values (202, 1, 40);

insert into OrePrice (id, description) values (203, 'Expensive Oxaria price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount) values (203, 2, 25);
insert into OrePriceAmount (orePriceId, oreId, amount) values (203, 1, 100);

-- Shop prices - Lithabine
insert into OrePrice (id, description) values (301, 'Lithabine price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount) values (301, 3, 10);
insert into OrePriceAmount (orePriceId, oreId, amount) values (301, 2, 30);
insert into OrePriceAmount (orePriceId, oreId, amount) values (301, 1, 60);

insert into OrePrice (id, description) values (302, 'Enhanced Lithabine price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount) values (302, 3, 25);
insert into OrePriceAmount (orePriceId, oreId, amount) values (302, 2, 50);
insert into OrePriceAmount (orePriceId, oreId, amount) values (302, 1, 120);

-- Shop prices - Neudralion
insert into OrePrice (id, description) values (401, 'Neudralion price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount) values (401, 4, 15);
insert into OrePriceAmount (orePriceId, oreId, amount) values (401, 3, 40);
insert into OrePriceAmount (orePriceId, oreId, amount) values (401, 2, 100);

insert into OrePrice (id, description) values (402, 'Enhanced Neudralion Ore Container price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount) values (402, 4, 50);
insert into OrePriceAmount (orePriceId, oreId, amount) values (402, 3, 100);
insert into OrePriceAmount (orePriceId, oreId, amount) values (402, 2, 200);

-- Shop prices - Complatix
insert into OrePrice (id, description) values (501, 'Complatix price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount) values (501, 5, 40);
insert into OrePriceAmount (orePriceId, oreId, amount) values (501, 4, 60);
insert into OrePriceAmount (orePriceId, oreId, amount) values (501, 3, 200);

insert into OrePrice (id, description) values (502, 'Enhanced Complatix price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount) values (502, 5, 80);
insert into OrePriceAmount (orePriceId, oreId, amount) values (502, 4, 150);
insert into OrePriceAmount (orePriceId, oreId, amount) values (502, 3, 500);

-- Shop prices - Prantum
insert into OrePrice (id, description) values (601, 'Prantum price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount) values (601, 6, 100);
insert into OrePriceAmount (orePriceId, oreId, amount) values (601, 5, 200);
insert into OrePriceAmount (orePriceId, oreId, amount) values (601, 4, 300);

insert into OrePrice (id, description) values (602, 'Enhanced Prantum price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount) values (602, 6, 200);
insert into OrePriceAmount (orePriceId, oreId, amount) values (602, 5, 400);
insert into OrePriceAmount (orePriceId, oreId, amount) values (602, 4, 600);

-- Shop prices - Raxia
insert into OrePrice (id, description) values (701, 'Raxia price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount) values (701, 7, 250);
insert into OrePriceAmount (orePriceId, oreId, amount) values (701, 6, 300);

insert into OrePrice (id, description) values (702, 'Enhanced Raxia price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount) values (702, 7, 400);
insert into OrePriceAmount (orePriceId, oreId, amount) values (702, 6, 500);

-- Shop prices - Dipolir
insert into OrePrice (id, description) values (801, 'Dipolir price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount) values (801, 8, 100);
insert into OrePriceAmount (orePriceId, oreId, amount) values (801, 7, 400);
insert into OrePriceAmount (orePriceId, oreId, amount) values (801, 6, 750);

insert into OrePrice (id, description) values (802, 'Enhanced Dipolir price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount) values (802, 8, 200);
insert into OrePriceAmount (orePriceId, oreId, amount) values (802, 7, 600);
insert into OrePriceAmount (orePriceId, oreId, amount) values (802, 6, 1000);

-- Shop prices - Asradon
insert into OrePrice (id, description) values (901, 'Asradon price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount) values (901, 9, 150);
insert into OrePriceAmount (orePriceId, oreId, amount) values (901, 8, 750);
insert into OrePriceAmount (orePriceId, oreId, amount) values (901, 7, 1500);

insert into OrePrice (id, description) values (902, 'Enhanced Asradon price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount) values (902, 9, 300);
insert into OrePriceAmount (orePriceId, oreId, amount) values (902, 8, 1000);
insert into OrePriceAmount (orePriceId, oreId, amount) values (902, 7, 2500);

-- Shop prices - Baratiem
insert into OrePrice (id, description) values (1001, 'Baratiem price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount) values (1001, 10, 200);
insert into OrePriceAmount (orePriceId, oreId, amount) values (1001,  9, 1250);
insert into OrePriceAmount (orePriceId, oreId, amount) values (1001,  8, 2000);

insert into OrePrice (id, description) values (1002, 'Enhanced Baratiem price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount) values (1002, 10, 400);
insert into OrePriceAmount (orePriceId, oreId, amount) values (1002,  9, 2500);
insert into OrePriceAmount (orePriceId, oreId, amount) values (1002,  8, 5000);

-- Shop prices - Etaxy
insert into OrePrice (id, description) values (1101, 'Etaxy price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount) values (1101, 11, 300);
insert into OrePriceAmount (orePriceId, oreId, amount) values (1101, 10, 3000);
insert into OrePriceAmount (orePriceId, oreId, amount) values (1101,  9, 5000);

insert into OrePrice (id, description) values (1102, 'Etaxy Baratiem price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount) values (1102, 11, 1000);
insert into OrePriceAmount (orePriceId, oreId, amount) values (1102, 10, 5000);
insert into OrePriceAmount (orePriceId, oreId, amount) values (1102,  9, 8000);


-- Ore containers - Cerbonium
insert into RobotPart (id,  typeId, partName,                     orePriceId, oreCapacity, weight, volume, powerUsage)
               values (101, 1,      'Standard Ore Container',     101,        2,           3,      3,      1),
                      (102, 1,      'Enhanced Ore Container',     102,        5,           6,      6,      2),
                      (103, 1,      'Cerbonium-XL Ore Container', 103,        7,           8,      8,      3)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), oreCapacity = VALUES(oreCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Ore containers - Oxaria
insert into RobotPart (id,  typeId, partName,                        orePriceId, oreCapacity, weight, volume, powerUsage)
               values (110, 1,      'Oxaria Ore Container',          201,        10,          11,     11,     4),
                      (111, 1,      'Oxaria-XL Ore Container',       202,        14,          15,     15,     5)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), oreCapacity = VALUES(oreCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Ore containers - Lithabine
insert into RobotPart (id,  typeId, partName,                           orePriceId, oreCapacity, weight, volume, powerUsage)
               values (120, 1,      'Lithabine Ore Container',          301,        19,          20,     20,     6),
                      (121, 1,      'Lithabine-XL Ore Container',       302,        25,          26,     26,     7)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), oreCapacity = VALUES(oreCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Ore containers - Neudralion
insert into RobotPart (id,  typeId, partName,                            orePriceId, oreCapacity, weight, volume, powerUsage)
               values (130, 1,      'Neudralion Ore Container',          401,        30,          31,     31,     8),
                      (131, 1,      'Neudralion-XL Ore Container',       402,        40,          41,     41,     9)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), oreCapacity = VALUES(oreCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Ore containers - Complatix
insert into RobotPart (id,  typeId, partName,                           orePriceId, oreCapacity, weight, volume, powerUsage)
               values (140, 1,      'Complatix Ore Container',          501,        50,          51,     51,     10),
                      (141, 1,      'Complatix-XL Ore Container',       502,        60,          61,     61,     11)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), oreCapacity = VALUES(oreCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Ore containers - Prantum
insert into RobotPart (id,  typeId, partName,                         orePriceId, oreCapacity, weight, volume, powerUsage)
               values (150, 1,      'Prantum Ore Container',          601,        70,          71,     71,     12),
                      (151, 1,      'Prantum-XL Ore Container',       602,        80,          81,     81,     13)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), oreCapacity = VALUES(oreCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Ore containers - Raxia
insert into RobotPart (id,  typeId, partName,                       orePriceId, oreCapacity, weight, volume, powerUsage)
               values (160, 1,      'Raxia Ore Container',          701,        90,          91,     91,     14),
                      (161, 1,      'Raxia-XL Ore Container',       702,        100,         101,    101,    15)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), oreCapacity = VALUES(oreCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Ore containers - Dipolir
insert into RobotPart (id,  typeId, partName,                         orePriceId, oreCapacity, weight, volume, powerUsage)
               values (170, 1,      'Dipolir Ore Container',          801,        110,         111,    111,    16),
                      (171, 1,      'Dipolir-XL Ore Container',       802,        120,         121,    121,    17)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), oreCapacity = VALUES(oreCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Ore containers - Asradon
insert into RobotPart (id,  typeId, partName,                         orePriceId, oreCapacity, weight, volume, powerUsage)
               values (180, 1,      'Asradon Ore Container',          901,        130,         131,    131,    18),
                      (181, 1,      'Asradon-XL Ore Container',       902,        140,         141,    141,    19)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), oreCapacity = VALUES(oreCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Ore containers - Baratiem
insert into RobotPart (id,  typeId, partName,                          orePriceId, oreCapacity, weight, volume, powerUsage)
               values (190, 1,      'Baratiem Ore Container',          1001,       150,         151,    151,    20),
                      (191, 1,      'Baratiem-XL Ore Container',       1002,       160,         161,    161,    21)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), oreCapacity = VALUES(oreCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Ore containers - Etaxy
insert into RobotPart (id,   typeId, partName,                       orePriceId, oreCapacity, weight, volume, powerUsage)
               values (1100, 1,      'Etaxy Ore Container',          1101,       170,         171,    171,    22),
                      (1101, 1,      'Etaxy-XL Ore Container',       1102,       180,         181,    181,    23)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), oreCapacity = VALUES(oreCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);


-- Mining units - Cerbonium
insert into RobotPart (id,  typeId, partName,                orePriceId, miningCapacity, weight, volume, powerUsage)
               values (201, 2,      'Standard Mining Unit',  101,        1,              2,      2,      1),
                      (202, 2,      'Fast Mining Unit',      102,        2,              3,      3,      3),
                      (203, 2,      'Efficient Mining Unit', 103,        2,              3,      3,      2)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), miningCapacity = VALUES(miningCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Mining units - Oxaria
insert into RobotPart (id,  typeId, partName,                       orePriceId, miningCapacity, weight, volume, powerUsage)
               values (210, 2,      'Oxaria Mining Unit',           201,        3,              4,      4,      4),
                      (211, 2,      'Efficient Oxaria Mining Unit', 202,        3,              4,      4,      3)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), miningCapacity = VALUES(miningCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Mining units - Lithabine
insert into RobotPart (id,  typeId, partName,                          orePriceId, miningCapacity, weight, volume, powerUsage)
               values (220, 2,      'Lithabine Mining Unit',           301,        4,              5,      5,      5),
                      (221, 2,      'Efficient Lithabine Mining Unit', 302,        4,              5,      5,      4)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), miningCapacity = VALUES(miningCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Mining units - Neudralion
insert into RobotPart (id,  typeId, partName,                           orePriceId, miningCapacity, weight, volume, powerUsage)
               values (230, 2,      'Neudralion Mining Unit',           401,        5,              6,      6,      6),
                      (231, 2,      'Efficient Neudralion Mining Unit', 402,        5,              6,      6,      5)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), miningCapacity = VALUES(miningCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Mining units - Complatix
insert into RobotPart (id,  typeId, partName,                          orePriceId, miningCapacity, weight, volume, powerUsage)
               values (240, 2,      'Complatix Mining Unit',           501,        6,              7,      7,      7),
                      (241, 2,      'Efficient Complatix Mining Unit', 502,        6,              7,      7,      6)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), miningCapacity = VALUES(miningCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Mining units - Prantum
insert into RobotPart (id,  typeId, partName,                        orePriceId, miningCapacity, weight, volume, powerUsage)
               values (250, 2,      'Prantum Mining Unit',           601,        7,              8,      8,      8),
                      (251, 2,      'Efficient Prantum Mining Unit', 602,        7,              8,      8,      7)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), miningCapacity = VALUES(miningCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Mining units - Raxia
insert into RobotPart (id,  typeId, partName,                      orePriceId, miningCapacity, weight, volume, powerUsage)
               values (260, 2,      'Raxia Mining Unit',           701,        8,              9,      9,      9),
                      (261, 2,      'Efficient Raxia Mining Unit', 702,        8,              9,      9,      8)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), miningCapacity = VALUES(miningCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Mining units - Dipolir
insert into RobotPart (id,  typeId, partName,                        orePriceId, miningCapacity, weight, volume, powerUsage)
               values (270, 2,      'Dipolir Mining Unit',           801,        9,              10,     10,     10),
                      (271, 2,      'Efficient Dipolir Mining Unit', 802,        9,              10,     10,     9)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), miningCapacity = VALUES(miningCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Mining units - Asradon
insert into RobotPart (id,  typeId, partName,                        orePriceId, miningCapacity, weight, volume, powerUsage)
               values (280, 2,      'Asradon Mining Unit',           901,        10,             11,     11,     11),
                      (281, 2,      'Efficient Asradon Mining Unit', 902,        10,             11,     11,     10)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), miningCapacity = VALUES(miningCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Mining units - Baratiem
insert into RobotPart (id,  typeId, partName,                         orePriceId, miningCapacity, weight, volume, powerUsage)
               values (290, 2,      'Baratiem Mining Unit',           1001,       11,             12,     12,     12),
                      (291, 2,      'Efficient Baratiem Mining Unit', 1002,       11,             12,     12,     11)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), miningCapacity = VALUES(miningCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Mining units - Etaxy
insert into RobotPart (id,   typeId, partName,                      orePriceId, miningCapacity, weight, volume, powerUsage)
               values (2100, 2,      'Etaxy Mining Unit',           1101,       12,             12,     12,     12),
                      (2101, 2,      'Efficient Etaxy Mining Unit', 1102,       12,             12,     12,     11)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), miningCapacity = VALUES(miningCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);


-- Batteries - Cerbonium
insert into RobotPart (id,  typeId, partName,            orePriceId, batteryCapacity, rechargeTime, weight, volume, powerUsage)
               values (301, 3,      'Standard Battery',  101,        140,             5,            10,     10,     0),
                      (302, 3,      'Enhanced Battery',  102,        230,             6,            12,     12,     0),
                      (303, 3,      'Cerbonium Battery', 103,        420,             7,            14,     14,     0)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), batteryCapacity = VALUES(batteryCapacity), rechargeTime = VALUES(rechargeTime), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Batteries - Oxaria
insert into RobotPart (id,  typeId, partName,                     orePriceId, batteryCapacity, rechargeTime, weight, volume, powerUsage)
               values (310, 3,      'Oxaria Battery',             201,        950,             10,           16,     15,     0),
                      (311, 3,      'Enhanced Oxaria Battery',    202,        2250,            20,           18,     16,     0),
                      (312, 3,      'Fast Charge Oxaria Battery', 203,        2000,            15,           19,     17,     0)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), batteryCapacity = VALUES(batteryCapacity), rechargeTime = VALUES(rechargeTime), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Batteries - Lithabine
insert into RobotPart (id,  typeId, partName,                     orePriceId, batteryCapacity, rechargeTime, weight, volume, powerUsage)
               values (320, 3,      'Lithabine Battery',          301,        4000,            25,           21,     20,      0),
                      (321, 3,      'Enhanced Lithabine Battery', 302,        6000,            30,           22,     21,      0)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), batteryCapacity = VALUES(batteryCapacity), rechargeTime = VALUES(rechargeTime), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Batteries - Neudralion
insert into RobotPart (id,  typeId, partName,                      orePriceId, batteryCapacity, rechargeTime, weight, volume, powerUsage)
               values (330, 3,      'Neudralion Battery',          401,        9000,            40,           23,     22,      0),
                      (331, 3,      'Enhanced Neudralion Battery', 402,        15000,           55,           24,     23,      0)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), batteryCapacity = VALUES(batteryCapacity), rechargeTime = VALUES(rechargeTime), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Batteries - Complatix
insert into RobotPart (id,  typeId, partName,                     orePriceId, batteryCapacity, rechargeTime, weight, volume, powerUsage)
               values (340, 3,      'Complatix Battery',          501,        19000,           75,           25,     24,      0),
                      (341, 3,      'Enhanced Complatix Battery', 502,        24000,           90,           26,     25,      0)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), batteryCapacity = VALUES(batteryCapacity), rechargeTime = VALUES(rechargeTime), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Batteries - Prantum
insert into RobotPart (id,  typeId, partName,                   orePriceId, batteryCapacity, rechargeTime, weight, volume, powerUsage)
               values (350, 3,      'Prantum Battery',          601,        29500,           110,          27,     26,      0),
                      (351, 3,      'Enhanced Prantum Battery', 602,        43000,           140,          28,     27,      0)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), batteryCapacity = VALUES(batteryCapacity), rechargeTime = VALUES(rechargeTime), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Batteries - Raxia
insert into RobotPart (id,  typeId, partName,                 orePriceId, batteryCapacity, rechargeTime, weight, volume, powerUsage)
               values (360, 3,      'Raxia Battery',          701,        57000,           180,          29,     28,      0),
                      (361, 3,      'Enhanced Raxia Battery', 702,        78500,           220,          30,     29,      0)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), batteryCapacity = VALUES(batteryCapacity), rechargeTime = VALUES(rechargeTime), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Batteries - Dipolir
insert into RobotPart (id,  typeId, partName,                   orePriceId, batteryCapacity, rechargeTime, weight, volume, powerUsage)
               values (370, 3,      'Dipolir Battery',          801,        90000,           270,          31,     30,      0),
                      (371, 3,      'Enhanced Dipolir Battery', 802,        122000,          320,          32,     31,      0)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), batteryCapacity = VALUES(batteryCapacity), rechargeTime = VALUES(rechargeTime), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Batteries - Asradon
insert into RobotPart (id,  typeId, partName,                   orePriceId, batteryCapacity, rechargeTime, weight, volume, powerUsage)
               values (380, 3,      'Asradon Battery',          901,        145000,          380,          33,     32,      0),
                      (381, 3,      'Enhanced Asradon Battery', 902,        190000,          320,          34,     33,      0)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), batteryCapacity = VALUES(batteryCapacity), rechargeTime = VALUES(rechargeTime), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Batteries - Baratiem
insert into RobotPart (id,  typeId, partName,                    orePriceId, batteryCapacity, rechargeTime, weight, volume, powerUsage)
               values (390, 3,      'Baratiem Battery',          1001,       280000,          400,          35,     34,      0),
                      (391, 3,      'Enhanced Baratiem Battery', 1002,       460000,          450,          36,     35,      0)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), batteryCapacity = VALUES(batteryCapacity), rechargeTime = VALUES(rechargeTime), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Batteries - Etaxy
insert into RobotPart (id,   typeId, partName,                 orePriceId, batteryCapacity, rechargeTime, weight, volume, powerUsage)
               values (3100, 3,      'Etaxy Battery',          1101,       650000,          600,          37,     36,      0),
                      (3101, 3,      'Enhanced Etaxy Battery', 1102,       990000,          700,          38,     37,      0)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), batteryCapacity = VALUES(batteryCapacity), rechargeTime = VALUES(rechargeTime), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);


-- Memory modules - Cerbonium
insert into RobotPart (id,  typeId, partName,                 orePriceId, memoryCapacity, weight, volume, powerUsage)
               values (401, 4,      'Standard Memory Module', 101,        4,              1,      1,      1),
                      (402, 4,      'Enhanced Memory Module', 102,        8,              1,      1,      2),
                      (403, 4,      'Cerbonium Memory Module', 103,       16,             1,      1,      3)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), memoryCapacity = VALUES(memoryCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Memory modules - Oxaria
insert into RobotPart (id,  typeId, partName,                        orePriceId, memoryCapacity, weight, volume, powerUsage)
               values (410, 4,      'Oxaria Memory Module',          201,        32,             1,      1,      4),
                      (411, 4,      'Enhanced Oxaria Memory Module', 202,        48,             1,      1,      5)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), memoryCapacity = VALUES(memoryCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Memory modules - Lithabine
insert into RobotPart (id,  typeId, partName,                           orePriceId, memoryCapacity, weight, volume, powerUsage)
               values (420, 4,      'Lithabine Memory Module',          301,        64,             1,      1,      6),
                      (421, 4,      'Enhanced Lithabine Memory Module', 302,        96,             1,      1,      7)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), memoryCapacity = VALUES(memoryCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Memory modules - Neudralion
insert into RobotPart (id,  typeId, partName,                            orePriceId, memoryCapacity, weight, volume, powerUsage)
               values (430, 4,      'Neudralion Memory Module',          401,        128,            1,      1,      8),
                      (431, 4,      'Enhanced Neudralion Memory Module', 402,        192,            1,      1,      9)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), memoryCapacity = VALUES(memoryCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Memory modules - Complatix
insert into RobotPart (id,  typeId, partName,                           orePriceId, memoryCapacity, weight, volume, powerUsage)
               values (440, 4,      'Complatix Memory Module',          501,        256,            1,      1,      10),
                      (441, 4,      'Enhanced Complatix Memory Module', 502,        384,            1,      1,      11)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), memoryCapacity = VALUES(memoryCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Memory modules - Prantum
insert into RobotPart (id,  typeId, partName,                         orePriceId, memoryCapacity, weight, volume, powerUsage)
               values (450, 4,      'Prantum Memory Module',          601,        512,            1,      1,      12),
                      (451, 4,      'Enhanced Prantum Memory Module', 602,        768,            1,      1,      13)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), memoryCapacity = VALUES(memoryCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Memory modules - Raxia
insert into RobotPart (id,  typeId, partName,                       orePriceId, memoryCapacity, weight, volume, powerUsage)
               values (460, 4,      'Raxia Memory Module',          701,        1024,           1,      1,      14),
                      (461, 4,      'Enhanced Raxia Memory Module', 702,        1536,           1,      1,      15)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), memoryCapacity = VALUES(memoryCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Memory modules - Dipolir
insert into RobotPart (id,  typeId, partName,                         orePriceId, memoryCapacity, weight, volume, powerUsage)
               values (470, 4,      'Dipolir Memory Module',          801,        2048,           1,      1,      16),
                      (471, 4,      'Enhanced Dipolir Memory Module', 802,        3072,           1,      1,      17)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), memoryCapacity = VALUES(memoryCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Memory modules - Asradon
insert into RobotPart (id,  typeId, partName,                         orePriceId, memoryCapacity, weight, volume, powerUsage)
               values (480, 4,      'Asradon Memory Module',          901,        4096,           1,      1,      18),
                      (481, 4,      'Enhanced Asradon Memory Module', 902,        6144,           1,      1,      19)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), memoryCapacity = VALUES(memoryCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Memory modules - Baratiem
insert into RobotPart (id,  typeId, partName,                          orePriceId, memoryCapacity, weight, volume, powerUsage)
               values (490, 4,      'Baratiem Memory Module',          1001,       8192,            1,      1,      20),
                      (491, 4,      'Enhanced Baratiem Memory Module', 1002,       12288,           1,      1,      21)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), memoryCapacity = VALUES(memoryCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Memory modules - Etaxy
insert into RobotPart (id,   typeId, partName,                       orePriceId, memoryCapacity, weight, volume, powerUsage)
               values (4100, 4,      'Etaxy Memory Module',          1101,       16384,          1,      1,      22),
                      (4101, 4,      'Enhanced Etaxy Memory Module', 1102,       24576,          1,      1,      23)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), memoryCapacity = VALUES(memoryCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);


-- CPUs - Cerbonium
insert into RobotPart (id,  typeId, partName,        orePriceId, cpuCapacity, weight, volume, powerUsage)
               values (501, 5,      'Standard CPU',  101,        1,           1,      1,      1),
                      (502, 5,      'Fast CPU',      102,        2,           1,      1,      2),
                      (503, 5,      'Efficient CPU', 103,        3,           1,      1,      1)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), cpuCapacity = VALUES(cpuCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- CPUs - Oxaria
insert into RobotPart (id,  typeId, partName,               orePriceId, cpuCapacity, weight, volume, powerUsage)
               values (510, 5,      'Oxaria CPU',           201,        6,           1,      1,      3),
                      (511, 5,      'Efficient Oxaria CPU', 202,        5,           1,      1,      2)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), cpuCapacity = VALUES(cpuCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- CPUs - Lithabine
insert into RobotPart (id,  typeId, partName,                  orePriceId, cpuCapacity, weight, volume, powerUsage)
               values (520, 5,      'Lithabine CPU',           301,        9,           1,      1,      4),
                      (521, 5,      'Efficient Lithabine CPU', 302,        8,           1,      1,      3)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), cpuCapacity = VALUES(cpuCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- CPUs - Neudralion
insert into RobotPart (id,  typeId, partName,                   orePriceId, cpuCapacity, weight, volume, powerUsage)
               values (530, 5,      'Neudralion CPU',           401,        11,          1,      1,      5),
                      (531, 5,      'Efficient Neudralion CPU', 402,        10,          1,      1,      4)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), cpuCapacity = VALUES(cpuCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- CPUs - Complatix
insert into RobotPart (id,  typeId, partName,                  orePriceId, cpuCapacity, weight, volume, powerUsage)
               values (540, 5,      'Complatix CPU',           501,        13,          1,      1,      6),
                      (541, 5,      'Efficient Complatix CPU', 502,        12,          1,      1,      5)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), cpuCapacity = VALUES(cpuCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- CPUs - Prantum
insert into RobotPart (id,  typeId, partName,                orePriceId, cpuCapacity, weight, volume, powerUsage)
               values (550, 5,      'Prantum CPU',           601,        15,          1,      1,      7),
                      (551, 5,      'Efficient Prantum CPU', 602,        14,          1,      1,      6)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), cpuCapacity = VALUES(cpuCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- CPUs - Raxia
insert into RobotPart (id,  typeId, partName,              orePriceId, cpuCapacity, weight, volume, powerUsage)
               values (560, 5,      'Raxia CPU',           701,        17,          1,      1,      8),
                      (561, 5,      'Efficient Raxia CPU', 702,        16,          1,      1,      6)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), cpuCapacity = VALUES(cpuCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- CPUs - Dipolir
insert into RobotPart (id,  typeId, partName,                orePriceId, cpuCapacity, weight, volume, powerUsage)
               values (570, 5,      'Dipolir CPU',           801,        19,          1,      1,      9),
                      (571, 5,      'Efficient Dipolir CPU', 802,        18,          1,      1,      7)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), cpuCapacity = VALUES(cpuCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- CPUs - Asradon
insert into RobotPart (id,  typeId, partName,                orePriceId, cpuCapacity, weight, volume, powerUsage)
               values (580, 5,      'Asradon CPU',           901,        21,          1,      1,      10),
                      (581, 5,      'Efficient Asradon CPU', 902,        20,          1,      1,      7)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), cpuCapacity = VALUES(cpuCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- CPUs - Baratiem
insert into RobotPart (id,  typeId, partName,                 orePriceId, cpuCapacity, weight, volume, powerUsage)
               values (590, 5,      'Baratiem CPU',           1001,       23,          1,      1,      11),
                      (591, 5,      'Efficient Baratiem CPU', 1002,       22,          1,      1,      8)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), cpuCapacity = VALUES(cpuCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- CPUs - Etaxy
insert into RobotPart (id,   typeId, partName,              orePriceId, cpuCapacity, weight, volume, powerUsage)
               values (5100, 5,      'Etaxy CPU',           1101,       25,          1,      1,      12),
                      (5101, 5,      'Efficient Etaxy CPU', 1102,       24,          1,      1,      9)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), cpuCapacity = VALUES(cpuCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);


-- Engines - Cerbonium
insert into RobotPart (id,  typeId, partName,           orePriceId, forwardCapacity, backwardCapacity, rotateCapacity, weight, volume, powerUsage)
               values (601, 6,      'Standard Engine',  101,        11,              5,                55,              2,      2,      4),
                      (602, 6,      'Enhanced Engine',  102,        14,              6,                65,             3,      3,      5),
                      (603, 6,      'Cerbonium Engine', 103,        18,              8,                75,             4,      4,      6)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), forwardCapacity = VALUES(forwardCapacity), backwardCapacity = VALUES(backwardCapacity), rotateCapacity = VALUES(rotateCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Engines - Oxaria
insert into RobotPart (id,  typeId, partName,                 orePriceId, forwardCapacity, backwardCapacity, rotateCapacity, weight, volume, powerUsage)
               values (610, 6,      'Oxaria Engine',          201,        22,              15,               85,             5,      5,      7),
                      (611, 6,      'Powerful Oxaria Engine', 202,        26,              20,               95,             6,      6,      8)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), forwardCapacity = VALUES(forwardCapacity), backwardCapacity = VALUES(backwardCapacity), rotateCapacity = VALUES(rotateCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Engines - Lithabine
insert into RobotPart (id,  typeId, partName,                    orePriceId, forwardCapacity, backwardCapacity, rotateCapacity, weight, volume, powerUsage)
               values (620, 6,      'Lithabine Engine',          301,        40,              20,               110,            7,      7,      9),
                      (621, 6,      'Powerful Lithabine Engine', 302,        68,              24,               125,            8,      8,      10)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), forwardCapacity = VALUES(forwardCapacity), backwardCapacity = VALUES(backwardCapacity), rotateCapacity = VALUES(rotateCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Engines - Neudralion
insert into RobotPart (id,  typeId, partName,                     orePriceId, forwardCapacity, backwardCapacity, rotateCapacity, weight, volume, powerUsage)
               values (630, 6,      'Neudralion Engine',          401,        84,              32,               140,            8,      8,      11),
                      (631, 6,      'Powerful Neudralion Engine', 402,        120,             40,               150,            9,      9,      12)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), forwardCapacity = VALUES(forwardCapacity), backwardCapacity = VALUES(backwardCapacity), rotateCapacity = VALUES(rotateCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Engines - Complatix
insert into RobotPart (id,  typeId, partName,                    orePriceId, forwardCapacity, backwardCapacity, rotateCapacity, weight, volume, powerUsage)
               values (640, 6,      'Complatix Engine',          501,        146,             48,               160,            10,     10,     13),
                      (641, 6,      'Powerful Complatix Engine', 502,        182,             64,               175,            11,     11,     14)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), forwardCapacity = VALUES(forwardCapacity), backwardCapacity = VALUES(backwardCapacity), rotateCapacity = VALUES(rotateCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Engines - Prantum
insert into RobotPart (id,  typeId, partName,                    orePriceId, forwardCapacity, backwardCapacity, rotateCapacity, weight, volume, powerUsage)
               values (650, 6,      'Prantum Engine',            601,        228,             64,               180,            12,     12,     15),
                      (651, 6,      'Powerful Prantum Engine',   602,        340,             80,               200,            13,     13,     16)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), forwardCapacity = VALUES(forwardCapacity), backwardCapacity = VALUES(backwardCapacity), rotateCapacity = VALUES(rotateCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Engines - Raxia
insert into RobotPart (id,  typeId, partName,                    orePriceId, forwardCapacity, backwardCapacity, rotateCapacity, weight, volume, powerUsage)
               values (660, 6,      'Raxia Engine',              701,        456,             78,               210,            14,     14,     17),
                      (661, 6,      'Powerful Raxia Engine',     702,        572,             100,              220,            15,     15,     18)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), forwardCapacity = VALUES(forwardCapacity), backwardCapacity = VALUES(backwardCapacity), rotateCapacity = VALUES(rotateCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Engines - Dipolir
insert into RobotPart (id,  typeId, partName,                    orePriceId, forwardCapacity, backwardCapacity, rotateCapacity, weight, volume, powerUsage)
               values (670, 6,      'Dipolir Engine',            801,        688,             94,               230,            16,     16,     19),
                      (671, 6,      'Powerful Dipolir Engine',   802,        804,             100,              240,            17,     17,     20)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), forwardCapacity = VALUES(forwardCapacity), backwardCapacity = VALUES(backwardCapacity), rotateCapacity = VALUES(rotateCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Engines - Asradon
insert into RobotPart (id,  typeId, partName,                    orePriceId, forwardCapacity, backwardCapacity, rotateCapacity, weight, volume, powerUsage)
               values (680, 6,      'Asradon Engine',            901,        920,             110,              242,            18,     18,     21),
                      (681, 6,      'Powerful Asradon Engine',   902,        1026,            118,              258,            19,     19,     22)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), forwardCapacity = VALUES(forwardCapacity), backwardCapacity = VALUES(backwardCapacity), rotateCapacity = VALUES(rotateCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Engines - Baratiem
insert into RobotPart (id,  typeId, partName,                    orePriceId, forwardCapacity, backwardCapacity, rotateCapacity, weight, volume, powerUsage)
               values (690, 6,      'Baratiem Engine',           1001,       1252,            126,              274,            20,     20,     23),
                      (691, 6,      'Powerful Baratiem Engine',  1002,       1468,            134,              290,            21,     21,     24)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), forwardCapacity = VALUES(forwardCapacity), backwardCapacity = VALUES(backwardCapacity), rotateCapacity = VALUES(rotateCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Engines - Etaxy
insert into RobotPart (id,   typeId, partName,                    orePriceId, forwardCapacity, backwardCapacity, rotateCapacity, weight, volume, powerUsage)
               values (6100, 6,      'Etaxy Engine',              1101,       1684,            142,              306,            26,     26,     29),
                      (6101, 6,      'Powerful Etaxy Engine',     1102,       1900,            150,              322,            27,     27,     30)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), forwardCapacity = VALUES(forwardCapacity), backwardCapacity = VALUES(backwardCapacity), rotateCapacity = VALUES(rotateCapacity), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);


-- Ore scanners - Cerbonium
insert into RobotPart (id,  typeId, partName,               orePriceId, scanTime, scanDistance, weight, volume, powerUsage)
               values (701, 7,      'Standard Ore Scanner', 101,        1,        1,            2,      2,      1),
                      (702, 7,      'Enhanced Ore Scanner', 102,        2,        2,            3,      3,      2),
                      (703, 7,      'Cerbonium Ore Scanner', 103,       2,        3,            4,      4,      3)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), scanTime = VALUES(scanTime), scanDistance = VALUES(scanDistance), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Ore scanners - Oxaria
insert into RobotPart (id,  typeId, partName,                      orePriceId, scanTime, scanDistance, weight, volume, powerUsage)
               values (710, 7,      'Oxaria Ore Scanner',          201,        3,        4,            5,      5,      4),
                      (711, 7,      'Enhanced Oxaria Ore Scanner', 202,        3,        5,            6,      6,      5)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), scanTime = VALUES(scanTime), scanDistance = VALUES(scanDistance), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Ore scanners - Lithabine
insert into RobotPart (id,  typeId, partName,                         orePriceId, scanTime, scanDistance, weight, volume, powerUsage)
               values (720, 7,      'Lithabine Ore Scanner',          301,        4,        6,            7,      7,     6),
                      (721, 7,      'Enhanced Lithabine Ore Scanner', 302,        5,        7,            8,      8,     7)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), scanTime = VALUES(scanTime), scanDistance = VALUES(scanDistance), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Ore scanners - Neudralion
insert into RobotPart (id,  typeId, partName,                          orePriceId, scanTime, scanDistance, weight, volume, powerUsage)
               values (730, 7,      'Neudralion Ore Scanner',          401,        5,        8,            9,      9,      8),
                      (731, 7,      'Enhanced Neudralion Ore Scanner', 402,        6,        9,            10,     10,     9)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), scanTime = VALUES(scanTime), scanDistance = VALUES(scanDistance), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Ore scanners - Complatix
insert into RobotPart (id,  typeId, partName,                         orePriceId, scanTime, scanDistance, weight, volume, powerUsage)
               values (740, 7,      'Complatix Ore Scanner',          501,        6,        10,           11,     11,     10),
                      (741, 7,      'Enhanced Complatix Ore Scanner', 502,        7,        11,           12,     12,     11)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), scanTime = VALUES(scanTime), scanDistance = VALUES(scanDistance), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Ore scanners - Prantum
insert into RobotPart (id,  typeId, partName,                         orePriceId, scanTime, scanDistance, weight, volume, powerUsage)
               values (750, 7,      'Prantum Ore Scanner',            601,        7,        12,           13,     13,     12),
                      (751, 7,      'Enhanced Prantum Ore Scanner',   602,        8,        13,           14,     14,     13)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), scanTime = VALUES(scanTime), scanDistance = VALUES(scanDistance), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Ore scanners - Raxia
insert into RobotPart (id,  typeId, partName,                         orePriceId, scanTime, scanDistance, weight, volume, powerUsage)
               values (760, 7,      'Raxia Ore Scanner',              701,        8,        14,           15,     15,     14),
                      (761, 7,      'Enhanced Raxia Ore Scanner',     702,        9,        15,           16,     16,     15)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), scanTime = VALUES(scanTime), scanDistance = VALUES(scanDistance), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Ore scanners - Dipolir
insert into RobotPart (id,  typeId, partName,                         orePriceId, scanTime, scanDistance, weight, volume, powerUsage)
               values (770, 7,      'Dipolir Ore Scanner',            801,        9,        16,           17,     17,     16),
                      (771, 7,      'Enhanced Dipolir Ore Scanner',   802,        10,       17,           18,     18,     17)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), scanTime = VALUES(scanTime), scanDistance = VALUES(scanDistance), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Ore scanners - Asradon
insert into RobotPart (id,  typeId, partName,                         orePriceId, scanTime, scanDistance, weight, volume, powerUsage)
               values (780, 7,      'Asradon Ore Scanner',            901,        10,       18,           19,     19,     18),
                      (781, 7,      'Enhanced Asradon Ore Scanner',   902,        11,       19,           20,     20,     19)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), scanTime = VALUES(scanTime), scanDistance = VALUES(scanDistance), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Ore scanners - Baratiem
insert into RobotPart (id,  typeId, partName,                         orePriceId, scanTime, scanDistance, weight, volume, powerUsage)
               values (790, 7,      'Baratiem Ore Scanner',           1001,       11,       20,           21,     21,     20),
                      (791, 7,      'Enhanced Baratiem Ore Scanner',  1002,       12,       21,           22,     22,     21)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), scanTime = VALUES(scanTime), scanDistance = VALUES(scanDistance), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);

-- Ore scanners - Etaxy
insert into RobotPart (id,  typeId, partName,                         orePriceId, scanTime, scanDistance, weight, volume, powerUsage)
               values (7100, 7,      'Etaxy Ore Scanner',             1101,       12,       22,           23,     23,     22),
                      (7101, 7,      'Enhanced Etaxy Ore Scanner',    1102,       13,       23,           24,     24,     23)
ON DUPLICATE KEY UPDATE typeId = VALUES(typeId), partName = VALUES(partName), orePriceId = VALUES(orePriceId), scanTime = VALUES(scanTime), scanDistance = VALUES(scanDistance), weight = VALUES(weight), volume = VALUES(volume), powerUsage = VALUES(powerUsage);


-- AI player
insert into User (id, username, email, password) values (1, 'AI', '', '') ON DUPLICATE KEY UPDATE username = VALUES(username), email = VALUES(email), password = VALUES(password);

-- AI player robots
insert into Robot (id, userId, robotName, sourceCode,
 rechargeTime, maxOre, miningSpeed, maxTurns, cpuSpeed, forwardSpeed, backwardSpeed, rotateSpeed, robotSize)
values (1, 1, 'AI-1', 'move(1.5); while (mine());',
 0,            50,     2,           1500,     99,       2,            2,             25,          1.5) ON DUPLICATE KEY UPDATE userId = VALUES(userId), robotName = VALUES(robotName), sourceCode = VALUES(sourceCode), rechargeTime = VALUES(rechargeTime), maxOre = VALUES(maxOre), miningSpeed = VALUES(miningSpeed), maxTurns = VALUES(maxTurns), cpuSpeed = VALUES(cpuSpeed), forwardSpeed = VALUES(forwardSpeed), backwardSpeed = VALUES(backwardSpeed), rotateSpeed = VALUES(rotateSpeed), robotSize = VALUES(robotSize);

insert into Robot (id, userId, robotName, sourceCode,
 rechargeTime, maxOre, miningSpeed, maxTurns, cpuSpeed, forwardSpeed, backwardSpeed, rotateSpeed, robotSize)
values (2, 1, 'AI-2', 'if (move(1.5) >= 1) { while (mine()); } else { move(-1); rotate(20); }',
 0,            50,     2,           3000,     99,       2,            2,             25,          1.5) ON DUPLICATE KEY UPDATE userId = VALUES(userId), robotName = VALUES(robotName), sourceCode = VALUES(sourceCode), rechargeTime = VALUES(rechargeTime), maxOre = VALUES(maxOre), miningSpeed = VALUES(miningSpeed), maxTurns = VALUES(maxTurns), cpuSpeed = VALUES(cpuSpeed), forwardSpeed = VALUES(forwardSpeed), backwardSpeed = VALUES(backwardSpeed), rotateSpeed = VALUES(rotateSpeed), robotSize = VALUES(robotSize);

insert into Robot (id, userId, robotName,
 sourceCode,
 rechargeTime, maxOre, miningSpeed, maxTurns, cpuSpeed, forwardSpeed, backwardSpeed, rotateSpeed, robotSize)
values (3, 1, 'AI-3', 
'int rot = 0; while (true) { if (rot) { if (rot <= 90) { rotate(rot); } rot = rot - 10; } if (move(1.5) < 1) { move(-1); rotate(24); } while (mine()) { rot = 100; } }',
 0,            50,     2,           5000,     99,       2,            2,             25,          1.5) ON DUPLICATE KEY UPDATE userId = VALUES(userId), robotName = VALUES(robotName), sourceCode = VALUES(sourceCode), rechargeTime = VALUES(rechargeTime), maxOre = VALUES(maxOre), miningSpeed = VALUES(miningSpeed), maxTurns = VALUES(maxTurns), cpuSpeed = VALUES(cpuSpeed), forwardSpeed = VALUES(forwardSpeed), backwardSpeed = VALUES(backwardSpeed), rotateSpeed = VALUES(rotateSpeed), robotSize = VALUES(robotSize);

-- Mining areas

-- Cerbonium
insert into OrePrice (id, description) values (10001, 'Mining Area Cerbonium-mini price') ON DUPLICATE KEY UPDATE description = VALUES(description);
-- No OrePriceAmount values, Cerbonium-mini mining is deliberately free
insert into MiningArea (id,   areaName,         orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId)
                values (1001, 'Cerbonium-mini', 10001,      10,    10,    20,       5,          25,      1) ON DUPLICATE KEY UPDATE areaName = VALUES(areaName), orePriceId = VALUES(orePriceId), sizeX = VALUES(sizeX), sizeY = VALUES(sizeY), maxMoves = VALUES(maxMoves), miningTime = VALUES(miningTime), taxRate = VALUES(taxRate), aiRobotId = VALUES(aiRobotId);
insert into MiningAreaOreSupply (miningAreaId, oreId, supply, radius)
                         values (1001,         1,     4,      4);

insert into OrePrice (id, description) values (10002, 'Mining Area Cerbonium-Starter price') ON DUPLICATE KEY UPDATE description = VALUES(description);
-- No OrePriceAmount values, Cerbonium-Starter mining is deliberately free
insert into MiningArea (id,   areaName,            orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId)
                values (1002, 'Cerbonium-Starter', 10002,      15,    15,    40,       10,         20,      2) ON DUPLICATE KEY UPDATE areaName = VALUES(areaName), orePriceId = VALUES(orePriceId), sizeX = VALUES(sizeX), sizeY = VALUES(sizeY), maxMoves = VALUES(maxMoves), miningTime = VALUES(miningTime), taxRate = VALUES(taxRate), aiRobotId = VALUES(aiRobotId);
insert into MiningAreaOreSupply (miningAreaId, oreId, supply, radius)
                         values (1002,         1,     6,     6),
                                (1002,         1,     6,     4);

insert into OrePrice (id, description) values (10003, 'Mining Area Cerbonium-Advanced price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount)
                    values (10003,      1,     1);
insert into MiningArea (id,   areaName,             orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId)
                values (1003, 'Cerbonium-Advanced', 10003,      20,    20,    100,      15,         0,       3) ON DUPLICATE KEY UPDATE areaName = VALUES(areaName), orePriceId = VALUES(orePriceId), sizeX = VALUES(sizeX), sizeY = VALUES(sizeY), maxMoves = VALUES(maxMoves), miningTime = VALUES(miningTime), taxRate = VALUES(taxRate), aiRobotId = VALUES(aiRobotId);
insert into MiningAreaOreSupply (miningAreaId, oreId, supply, radius)
                         values (1003,         1,     8,      7),
                                (1003,         1,     6,      5);

-- Oxaria
insert into OrePrice (id, description) values (11001, 'Mining Area Oxaria-Light price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount)
                    values (11001,      1,     1);
insert into MiningArea (id,   areaName,       orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId)
                values (1101, 'Oxaria-Light', 11001,      20,    20,    60,       20,         25,      2) ON DUPLICATE KEY UPDATE areaName = VALUES(areaName), orePriceId = VALUES(orePriceId), sizeX = VALUES(sizeX), sizeY = VALUES(sizeY), maxMoves = VALUES(maxMoves), miningTime = VALUES(miningTime), taxRate = VALUES(taxRate), aiRobotId = VALUES(aiRobotId);
insert into MiningAreaOreSupply (miningAreaId, oreId, supply, radius)
                         values (1101,         1,     12,     6),
                                (1101,         2,     6,      4),
                                (1101,         2,     6,      4);

insert into OrePrice (id, description) values (11002, 'Mining Area Oxaria-Advanced price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount)
                    values (11002,      1,     2),
                           (11002,      2,     1);
insert into MiningArea (id,   areaName,          orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId)
                values (1102, 'Oxaria-Advanced', 11002,      25,    25,    150,      30,         10,      3) ON DUPLICATE KEY UPDATE areaName = VALUES(areaName), orePriceId = VALUES(orePriceId), sizeX = VALUES(sizeX), sizeY = VALUES(sizeY), maxMoves = VALUES(maxMoves), miningTime = VALUES(miningTime), taxRate = VALUES(taxRate), aiRobotId = VALUES(aiRobotId);
insert into MiningAreaOreSupply (miningAreaId, oreId, supply, radius)
                         values (1102,         1,     20,     6),
                                (1102,         2,     8,      4),
                                (1102,         2,     6,      4),
                                (1102,         2,     6,      4);

insert into OrePrice (id, description) values (11003, 'Mining Area Oxaria-Expert price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount)
                    values (11003,      1,     5),
                           (11003,      2,     1);
insert into MiningArea (id,   areaName,        orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId)
                values (1103, 'Oxaria-Expert', 11003,      30,    30,    200,      40,         0,       3) ON DUPLICATE KEY UPDATE areaName = VALUES(areaName), orePriceId = VALUES(orePriceId), sizeX = VALUES(sizeX), sizeY = VALUES(sizeY), maxMoves = VALUES(maxMoves), miningTime = VALUES(miningTime), taxRate = VALUES(taxRate), aiRobotId = VALUES(aiRobotId);
insert into MiningAreaOreSupply (miningAreaId, oreId, supply, radius)
                         values (1103,         1,     20,     6),
                                (1103,         2,     8,      4),
                                (1103,         2,     6,      4),
                                (1103,         2,     6,      4);

-- Lithabine
insert into OrePrice (id, description) values (12001, 'Mining Area Lithabine-Small price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount)
                    values (12001,      1,     10),
                           (12001,      2,     2);
insert into MiningArea (id,   areaName,          orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId)
                values (1201, 'Lithabine-Small', 12001,      35,    35,    200,      60,         25,      3) ON DUPLICATE KEY UPDATE areaName = VALUES(areaName), orePriceId = VALUES(orePriceId), sizeX = VALUES(sizeX), sizeY = VALUES(sizeY), maxMoves = VALUES(maxMoves), miningTime = VALUES(miningTime), taxRate = VALUES(taxRate), aiRobotId = VALUES(aiRobotId);
insert into MiningAreaOreSupply (miningAreaId, oreId, supply, radius)
                         values (1201,         1,     10,     8),
                                (1201,         2,     8,      4),
                                (1201,         3,     6,      4),
                                (1201,         3,     6,      4);

insert into OrePrice (id, description) values (12002, 'Mining Area Lithabine-Medium price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount)
                    values (12002,      1,     15),
                           (12002,      2,     5);
insert into MiningArea (id,   areaName,           orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId)
                values (1202, 'Lithabine-Medium', 12002,      40,    40,    400,      120,        10,      3) ON DUPLICATE KEY UPDATE areaName = VALUES(areaName), orePriceId = VALUES(orePriceId), sizeX = VALUES(sizeX), sizeY = VALUES(sizeY), maxMoves = VALUES(maxMoves), miningTime = VALUES(miningTime), taxRate = VALUES(taxRate), aiRobotId = VALUES(aiRobotId);
insert into MiningAreaOreSupply (miningAreaId, oreId, supply, radius)
                         values (1202,         1,     10,     8),
                                (1202,         2,     8,      4),
                                (1202,         3,     6,      4),
                                (1202,         3,     6,      4);

insert into OrePrice (id, description) values (12003, 'Mining Area Lithabine-Large price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount)
                    values (12003,      1,     20),
                           (12003,      2,     5),
                           (12003,      3,     1);
insert into MiningArea (id,   areaName,          orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId)
                values (1203, 'Lithabine-Large', 12003,      45,    45,    600,      180,        0,       3) ON DUPLICATE KEY UPDATE areaName = VALUES(areaName), orePriceId = VALUES(orePriceId), sizeX = VALUES(sizeX), sizeY = VALUES(sizeY), maxMoves = VALUES(maxMoves), miningTime = VALUES(miningTime), taxRate = VALUES(taxRate), aiRobotId = VALUES(aiRobotId);
insert into MiningAreaOreSupply (miningAreaId, oreId, supply, radius)
                         values (1203,         1,     20,     10),
                                (1203,         2,     10,     8),
                                (1203,         3,     6,      5),
                                (1203,         3,     6,      5);

-- Neudralion
insert into OrePrice (id, description) values (13001, 'Mining Area Neudralion-Small price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount)
                    values (13001,      2,     10),
                           (13001,      3,     5);
insert into MiningArea (id,   areaName,       orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId)
                values (1301, 'Neudralion-Small', 13001,      50,    50,    400,      300,        40,      3) ON DUPLICATE KEY UPDATE areaName = VALUES(areaName), orePriceId = VALUES(orePriceId), sizeX = VALUES(sizeX), sizeY = VALUES(sizeY), maxMoves = VALUES(maxMoves), miningTime = VALUES(miningTime), taxRate = VALUES(taxRate), aiRobotId = VALUES(aiRobotId);
insert into MiningAreaOreSupply (miningAreaId, oreId, supply, radius)
                         values (1301,         2,     10,     10),
                                (1301,         2,     10,     8),
                                (1301,         3,     8,      6),
                                (1301,         3,     8,      6),
                                (1301,         4,     5,      5),
                                (1301,         4,     5,      5);

insert into OrePrice (id, description) values (13002, 'Mining Area Neudralion-Large price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount)
                    values (13002,      2,     15),
                           (13002,      3,     10),
                           (13002,      4,     1);
insert into MiningArea (id,   areaName,           orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId)
                values (1302, 'Neudralion-Large', 13002,      70,    70,    600,      600,        20,      3) ON DUPLICATE KEY UPDATE areaName = VALUES(areaName), orePriceId = VALUES(orePriceId), sizeX = VALUES(sizeX), sizeY = VALUES(sizeY), maxMoves = VALUES(maxMoves), miningTime = VALUES(miningTime), taxRate = VALUES(taxRate), aiRobotId = VALUES(aiRobotId);
insert into MiningAreaOreSupply (miningAreaId, oreId, supply, radius)
                         values (1302,         2,     10,     8),
                                (1302,         3,     10,     5),
                                (1302,         4,     3,      4),
                                (1302,         4,     5,      5),
                                (1302,         4,     5,      5),
                                (1302,         4,     7,      6);

-- Complatix
insert into OrePrice (id, description) values (14001, 'Mining Area Complatix-Small price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount)
                    values (14001,      2,     20),
                           (14001,      3,     15),
                           (14001,      4,     10);
insert into MiningArea (id,   areaName,          orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId)
                values (1401, 'Complatix-Small', 14001,      60,    60,    600,      900,        50,      3) ON DUPLICATE KEY UPDATE areaName = VALUES(areaName), orePriceId = VALUES(orePriceId), sizeX = VALUES(sizeX), sizeY = VALUES(sizeY), maxMoves = VALUES(maxMoves), miningTime = VALUES(miningTime), taxRate = VALUES(taxRate), aiRobotId = VALUES(aiRobotId);
insert into MiningAreaOreSupply (miningAreaId, oreId, supply, radius)
                         values (1401,         3,     10,     8),
                                (1401,         4,     5,      5),
                                (1401,         4,     5,      5),
                                (1401,         5,     5,      5);

insert into OrePrice (id, description) values (14002, 'Mining Area Complatix-Large price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount)
                    values (14002,      3,     15),
                           (14002,      4,     10),
                           (14002,      5,     1);
insert into MiningArea (id,   areaName,          orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId)
                values (1402, 'Complatix-Large', 14002,      90,    90,    1200,     1800,       25,      3) ON DUPLICATE KEY UPDATE areaName = VALUES(areaName), orePriceId = VALUES(orePriceId), sizeX = VALUES(sizeX), sizeY = VALUES(sizeY), maxMoves = VALUES(maxMoves), miningTime = VALUES(miningTime), taxRate = VALUES(taxRate), aiRobotId = VALUES(aiRobotId);
insert into MiningAreaOreSupply (miningAreaId, oreId, supply, radius)
                         values (1402,         3,     10,     8),
                                (1402,         4,     5,      5),
                                (1402,         4,     5,      5),
                                (1402,         5,     5,      5);

-- Prantum
insert into OrePrice (id, description) values (15001, 'Mining Area Prantum-Small price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount)
                    values (15001,      3,     25),
                           (15001,      4,     20),
                           (15001,      5,     15);
insert into MiningArea (id,   areaName,        orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId)
                values (1501, 'Prantum-Small', 15001,      70,    70,    900,      1800,       60,      3) ON DUPLICATE KEY UPDATE areaName = VALUES(areaName), orePriceId = VALUES(orePriceId), sizeX = VALUES(sizeX), sizeY = VALUES(sizeY), maxMoves = VALUES(maxMoves), miningTime = VALUES(miningTime), taxRate = VALUES(taxRate), aiRobotId = VALUES(aiRobotId);
insert into MiningAreaOreSupply (miningAreaId, oreId, supply, radius)
                         values (1501,         3,     15,     15),
                                (1501,         4,     15,     15),
                                (1501,         6,     4,      5),
                                (1501,         6,     4,      5);

insert into OrePrice (id, description) values (15002, 'Mining Area Prantum-Large price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount)
                    values (15002,      4,     30),
                           (15002,      5,     20),
                           (15002,      6,     5);
insert into MiningArea (id,   areaName,        orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId)
                values (1502, 'Prantum-Large', 15002,      100,   100,   1500,     2700,       10,      3) ON DUPLICATE KEY UPDATE areaName = VALUES(areaName), orePriceId = VALUES(orePriceId), sizeX = VALUES(sizeX), sizeY = VALUES(sizeY), maxMoves = VALUES(maxMoves), miningTime = VALUES(miningTime), taxRate = VALUES(taxRate), aiRobotId = VALUES(aiRobotId);
insert into MiningAreaOreSupply (miningAreaId, oreId, supply, radius)
                         values (1502,         3,     15,     15),
                                (1502,         4,     15,     15),
                                (1502,         6,     4,      5),
                                (1502,         6,     4,      5);

-- Raxia
insert into OrePrice (id, description) values (16001, 'Mining Area Raxia-Small price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount)
                    values (16001,      4,     30),
                           (16001,      5,     25),
                           (16001,      6,     20);
insert into MiningArea (id,   areaName,      orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId)
                values (1601, 'Raxia-Small', 16001,      80,    80,    1250,     3600,       65,      3) ON DUPLICATE KEY UPDATE areaName = VALUES(areaName), orePriceId = VALUES(orePriceId), sizeX = VALUES(sizeX), sizeY = VALUES(sizeY), maxMoves = VALUES(maxMoves), miningTime = VALUES(miningTime), taxRate = VALUES(taxRate), aiRobotId = VALUES(aiRobotId);
insert into MiningAreaOreSupply (miningAreaId, oreId, supply, radius)
                         values (1601,         3,     15,     15),
                                (1601,         3,     15,     15),
                                (1601,         5,     15,     15),
                                (1601,         7,     4,      4),
                                (1601,         7,     4,      4);

insert into OrePrice (id, description) values (16002, 'Mining Area Raxia-Large price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount)
                    values (16002,      5,     40),
                           (16002,      6,     30),
                           (16002,      7,     25);
insert into MiningArea (id,   areaName,      orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId)
                values (1602, 'Raxia-Large', 16002,      110,   110,   1500,     4800,       5,       3) ON DUPLICATE KEY UPDATE areaName = VALUES(areaName), orePriceId = VALUES(orePriceId), sizeX = VALUES(sizeX), sizeY = VALUES(sizeY), maxMoves = VALUES(maxMoves), miningTime = VALUES(miningTime), taxRate = VALUES(taxRate), aiRobotId = VALUES(aiRobotId);
insert into MiningAreaOreSupply (miningAreaId, oreId, supply, radius)
                         values (1602,         3,     15,     15),
                                (1602,         3,     15,     15),
                                (1602,         5,     15,     15),
                                (1602,         7,     4,      4),
                                (1602,         7,     4,      4);

-- Dipolir
insert into OrePrice (id, description) values (17001, 'Mining Area Dipolir-Small price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount)
                    values (17001,      5,     20),
                           (17001,      6,     15),
                           (17001,      7,     10);
insert into MiningArea (id,   areaName,        orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId)
                values (1701, 'Dipolir-Small', 17001,      90,    90,    1800,     7200,       60,      3) ON DUPLICATE KEY UPDATE areaName = VALUES(areaName), orePriceId = VALUES(orePriceId), sizeX = VALUES(sizeX), sizeY = VALUES(sizeY), maxMoves = VALUES(maxMoves), miningTime = VALUES(miningTime), taxRate = VALUES(taxRate), aiRobotId = VALUES(aiRobotId);
insert into MiningAreaOreSupply (miningAreaId, oreId, supply, radius)
                         values (1701,         2,     15,     15),
                                (1701,         2,     15,     15),
                                (1701,         4,     15,     15),
                                (1701,         8,     3,      4),
                                (1701,         8,     3,      4),
                                (1701,         8,     3,      4);

-- Asradon
insert into OrePrice (id, description) values (18001, 'Mining Area Asradon-Small price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount)
                    values (18001,      6,     20),
                           (18001,      7,     15),
                           (18001,      8,     10);
insert into MiningArea (id,   areaName,        orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId)
                values (1801, 'Asradon-Small', 18001,      100,   100,   2700,     10800,      65,      3) ON DUPLICATE KEY UPDATE areaName = VALUES(areaName), orePriceId = VALUES(orePriceId), sizeX = VALUES(sizeX), sizeY = VALUES(sizeY), maxMoves = VALUES(maxMoves), miningTime = VALUES(miningTime), taxRate = VALUES(taxRate), aiRobotId = VALUES(aiRobotId);
insert into MiningAreaOreSupply (miningAreaId, oreId, supply, radius)
                         values (1801,         2,     15,     15),
                                (1801,         2,     15,     15),
                                (1801,         3,     15,     15),
                                (1801,         9,     3,      4),
                                (1801,         9,     3,      4),
                                (1801,         9,     3,      4);

-- Baratiem
insert into OrePrice (id, description) values (19001, 'Mining Area Baratiem-Small price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount)
                    values (19001,      7,     20),
                           (19001,      8,     15),
                           (19001,      9,     10);
insert into MiningArea (id,   areaName,         orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId)
                values (1901, 'Baratiem-Small', 19001,      110,   110,   3200,     14400,      70,      3) ON DUPLICATE KEY UPDATE areaName = VALUES(areaName), orePriceId = VALUES(orePriceId), sizeX = VALUES(sizeX), sizeY = VALUES(sizeY), maxMoves = VALUES(maxMoves), miningTime = VALUES(miningTime), taxRate = VALUES(taxRate), aiRobotId = VALUES(aiRobotId);
insert into MiningAreaOreSupply (miningAreaId, oreId, supply, radius)
                         values (1901,         1,     15,     15),
                                (1901,         1,     15,     15),
                                (1901,         2,     15,     15),
                                (1901,        10,     3,      4),
                                (1901,        10,     3,      4),
                                (1901,        10,     3,      4);

-- Etaxy
insert into OrePrice (id, description) values (20001, 'Mining Area Etaxy-Small price') ON DUPLICATE KEY UPDATE description = VALUES(description);
insert into OrePriceAmount (orePriceId, oreId, amount)
                    values (20001,      8,     15),
                           (20001,      9,     10),
                           (20001,     10,     1);
insert into MiningArea (id,   areaName,      orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId)
                values (2001, 'Etaxy-Small', 20001,      120,   120,   3500,     21600,      75,      3) ON DUPLICATE KEY UPDATE areaName = VALUES(areaName), orePriceId = VALUES(orePriceId), sizeX = VALUES(sizeX), sizeY = VALUES(sizeY), maxMoves = VALUES(maxMoves), miningTime = VALUES(miningTime), taxRate = VALUES(taxRate), aiRobotId = VALUES(aiRobotId);
insert into MiningAreaOreSupply (miningAreaId, oreId, supply, radius)
                         values (2001,         1,     15,     15),
                                (2001,         1,     15,     15),
                                (2001,         2,     15,     15),
                                (2001,        11,     3,      4),
                                (2001,        11,     3,      4),
                                (2001,        11,     3,      4);


-- Achievements - Initial achievement
insert into Achievement (id, title,              description)
                 values (1,  'Your first robot', 'Claim your first robot')
ON DUPLICATE KEY UPDATE title = VALUES(title), description = VALUES(description);

insert into AchievementStep (achievementId, step, achievementPoints, miningQueueReward, robotReward, miningAreaId)
                     values (1,             1,    10,                1,                 1,           1001);


-- Achievements - Cerbonium Mastery
insert into Achievement (id, title,               description)
                 values (2,  'Cerbonium Mastery', 'Master mining in the Cerbonium areas')
ON DUPLICATE KEY UPDATE title = VALUES(title), description = VALUES(description);

insert into AchievementPredecessor (predecessorId, predecessorStep, successorId)
                            values (1,             1,               2);


insert into AchievementStep (achievementId, step, achievementPoints, miningQueueReward)
                     values (2,             1,    10,                1);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (2,             1,    1,     1);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward)
                     values (2,             2,    10,                1,     20);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (2,             2,    1,     20);

insert into AchievementStep (achievementId, step, achievementPoints, miningAreaId)
                     values (2,             3,    10,                1002);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (2,             3,    1001,         70.0);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (2,             3,    1,     25);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (2,             4,    10,                1,     50,           5);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (2,             4,    1,     50);

insert into AchievementStep (achievementId, step, achievementPoints, miningQueueReward)
                     values (2,             5,    10,                1);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (2,             5,    1,     75);

insert into AchievementStep (achievementId, step, achievementPoints, miningAreaId)
                     values (2,             6,    10,                1003);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (2,             6,    1002,         120.0);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (2,             6,    1,     100);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (2,             7,    10,                1,     100,          10);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (2,             7,    1,     120);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (2,             8,    10,                1,     500,          50);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (2,             8,    1001,         100.0),
                                                  (2,             8,    1002,         150.0),
                                                  (2,             8,    1003,         150.0);

insert into AchievementStep (achievementId, step, achievementPoints, miningQueueReward)
                     values (2,             9,    10,                1);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (2,             9,    1,     500);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (2,             10,   10,                1,     1000,         100);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (2,             10,   1,     1500);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (2,             11,   10,                1,     5000,         500);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (2,             11,   1001,         100.0),
                                                  (2,             11,   1002,         400.0),
                                                  (2,             11,   1003,         600.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (2,             12,   10,                1,     9999,         1000);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (2,             12,   1003,         900.0);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (2,             12,   1,     15000);


-- Achievements - Oxaria Mastery
insert into Achievement (id, title,            description)
                 values (3,  'Oxaria Mastery', 'Master mining in the Oxaria areas')
ON DUPLICATE KEY UPDATE title = VALUES(title), description = VALUES(description);

insert into AchievementPredecessor (predecessorId, predecessorStep, successorId)
                            values (2,             7,               3);

insert into AchievementStep (achievementId, step, achievementPoints, miningAreaId, oreId, maxOreReward, maxDepotReward)
                     values (3,             1,    10,                1101,         2,     10,           10);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (3,             1,    1,     125);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (3,             1,    1002,         125.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (3,             2,    10,                2,     20,           20);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (3,             2,    2,     30);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (3,             3,    10,                2,     50,           50);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (3,             3,    2,     75);

insert into AchievementStep (achievementId, step, achievementPoints, miningAreaId)
                     values (3,             4,    10,                1102);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (3,             4,    1101,         90.0);

insert into AchievementStep (achievementId, step, achievementPoints, miningQueueReward)
                     values (3,             5,    10,                1);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (3,             5,    2,     100);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (3,             6,    10,                2,     100,          100);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (3,             6,    2,     150);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (3,             7,    10,                2,     500,          500);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (3,             7,    1101,         200.0),
                                                  (3,             7,    1102,         200.0);

insert into AchievementStep (achievementId, step, achievementPoints, miningQueueReward)
                     values (3,             8,    10,                1);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (3,             8,    1101,         250.0),
                                                  (3,             8,    1102,         250.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (3,             9,    10,                2,     1000,         1000);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (3,             9,    1101,         275.0),
                                                  (3,             9,    1102,         400.0);

insert into AchievementStep (achievementId, step, achievementPoints, miningAreaId, miningQueueReward)
                     values (3,             10,   10,                1103,         1);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (3,             10,   1101,         300.0),
                                                  (3,             10,   1102,         500.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward)
                     values (3,             11,   10,                2,     5000);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (3,             11,   1101,         350.0),
                                                  (3,             11,   1102,         750.0),
                                                  (3,             11,   1103,         600.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward)
                     values (3,             12,   10,                2,     9999);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (3,             12,   1103,         900.0);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (3,             12,   2,     15000);


-- Achievements - Lithabine Mastery
insert into Achievement (id, title,               description)
                 values (4,  'Lithabine Mastery', 'Master mining in the Lithabine areas')
ON DUPLICATE KEY UPDATE title = VALUES(title), description = VALUES(description);

insert into AchievementPredecessor (predecessorId, predecessorStep, successorId)
                            values (3,             6,               4);

insert into AchievementStep (achievementId, step, achievementPoints, miningAreaId, oreId, maxOreReward, maxDepotReward)
                     values (4,             1,    10,                1201,         3,     15,           10);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (4,             1,    2,     125);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (4,             1,    1101,         90.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (4,             2,    10,                3,     30,           30);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (4,             2,    3,     50);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (4,             3,    10,                3,     50,           50);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (4,             3,    3,     75);

insert into AchievementStep (achievementId, step, achievementPoints, miningAreaId)
                     values (4,             4,    10,                1202);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (4,             4,    1201,         150.0);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (4,             4,    3,     90);

insert into AchievementStep (achievementId, step, achievementPoints, miningQueueReward)
                     values (4,             5,    10,                1);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (4,             5,    1201,         300.0),
                                                  (4,             5,    1202,         300.0);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (4,             5,    3,     150);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (4,             6,    10,                3,     100,          100);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (4,             6,    3,     250);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (4,             7,    10,                3,     500,          500);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (4,             7,    1201,         450.0),
                                                  (4,             7,    1202,         450.0);

insert into AchievementStep (achievementId, step, achievementPoints, miningAreaId)
                     values (4,             8,    10,                1203);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (4,             8,    1201,         500.0),
                                                  (4,             8,    1202,         500.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (4,             9,    10,                3,     1000,         1000);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (4,             9,    1201,         550.0),
                                                  (4,             9,    1202,         550.0),
                                                  (4,             9,    1203,         550.0);

insert into AchievementStep (achievementId, step, achievementPoints, miningQueueReward)
                     values (4,             10,   10,                1);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (4,             10,   1201,         650.0),
                                                  (4,             10,   1202,         650.0),
                                                  (4,             10,   1203,         650.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward)
                     values (4,             11,   10,                3,     5000);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (4,             11,   1201,         750.0),
                                                  (4,             11,   1202,         750.0),
                                                  (4,             11,   1203,         750.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward)
                     values (4,             12,   10,                3,     9999);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (4,             12,   1202,         900.0),
                                                  (4,             12,   1203,         900.0);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (4,             12,   3,     15000);


-- Achievements - Neudralion Mastery
insert into Achievement (id, title,               description)
                 values (5,  'Neudralion Mastery', 'Master mining in the Neudralion areas')
ON DUPLICATE KEY UPDATE title = VALUES(title), description = VALUES(description);

insert into AchievementPredecessor (predecessorId, predecessorStep, successorId)
                            values (4,             4,               5);

insert into AchievementStep (achievementId, step, achievementPoints, miningAreaId, oreId, maxOreReward, maxDepotReward)
                     values (5,             1,    10,                1301,         4,     20,           20);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (5,             1,    3,     200);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (5,             1,    1201,         350.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (5,             2,    10,                4,     40,           40);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (5,             2,    4,     60);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (5,             3,    10,                4,     75,           75);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (5,             3,    4,     150);

insert into AchievementStep (achievementId, step, achievementPoints, miningAreaId)
                     values (5,             4,    10,                1302);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (5,             4,    1301,         400.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (5,             5,    10,                4,     200,          200);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (5,             5,    4,     300);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (5,             5,    1301,         450.0),
                                                  (5,             5,    1302,         250.0);

insert into AchievementStep (achievementId, step, achievementPoints, miningQueueReward)
                     values (5,             6,    10,                1);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (5,             6,    1301,         500.0),
                                                  (5,             6,    1302,         400.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (5,             7,    10,                4,     500,          500);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (5,             7,    1301,         700.0),
                                                  (5,             7,    1302,         600.0);

insert into AchievementStep (achievementId, step, achievementPoints, miningQueueReward)
                     values (5,             8,    10,                1);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (5,             8,    1301,         750.0),
                                                  (5,             8,    1302,         700.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (5,             9,    10,                4,     1000,         1000);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (5,             9,    1301,         800.0),
                                                  (5,             9,    1302,         800.0);

insert into AchievementStep (achievementId, step, achievementPoints, miningQueueReward)
                     values (5,             10,   10,                1);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (5,             10,   1301,         850.0),
                                                  (5,             10,   1302,         800.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward)
                     values (5,             11,   10,                4,     5000);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (5,             11,   1301,         900.0),
                                                  (5,             11,   1302,         900.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward)
                     values (5,             12,   10,                4,     9999);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (5,             12,   1301,         950.0),
                                                  (5,             12,   1302,         950.0);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (5,             12,   4,     15000);


-- Achievements - More robots
insert into Achievement (id, title,         description)
                 values (6,  'More robots', 'Earn an extra robot')
ON DUPLICATE KEY UPDATE title = VALUES(title), description = VALUES(description);

insert into AchievementPredecessor (predecessorId, predecessorStep, successorId)
                            values (5,             5,               6);

insert into AchievementStep (achievementId, step, achievementPoints, robotReward)
                     values (6,             1,    10,                2);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (6,             1,    1,     4000),
                                                  (6,             1,    2,     3500),
                                                  (6,             1,    3,     3000),
                                                  (6,             1,    4,     2000);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (6,             1,    1003,         900.0),
                                                  (6,             1,    1103,         900.0),
                                                  (6,             1,    1203,         900.0),
                                                  (6,             1,    1302,         900.0);


-- Achievements - Complatix Mastery
insert into Achievement (id, title,               description)
                 values (7,  'Complatix Mastery', 'Master mining in the Complatix areas')
ON DUPLICATE KEY UPDATE title = VALUES(title), description = VALUES(description);

insert into AchievementPredecessor (predecessorId, predecessorStep, successorId)
                            values (5,             5,               7);

insert into AchievementStep (achievementId, step, achievementPoints, miningAreaId, oreId, maxOreReward, maxDepotReward)
                     values (7,             1,    10,                1401,         5,     50,           50);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (7,             1,    4,     500);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (7,             1,    1301,         400.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (7,             2,    10,                5,     80,           80);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (7,             2,    5,     250);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (7,             3,    10,                5,     150,          150);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (7,             3,    5,     400);

insert into AchievementStep (achievementId, step, achievementPoints, miningAreaId)
                     values (7,             4,    10,                1402);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (7,             4,    1401,         550.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (7,             5,    10,                5,     500,          500);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (7,             5,    5,     900);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (7,             5,    1401,         600.0),
                                                  (7,             5,    1402,         400.0);

insert into AchievementStep (achievementId, step, achievementPoints, miningQueueReward)
                     values (7,             6,    10,                1);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (7,             6,    1401,         650.0),
                                                  (7,             6,    1402,         500.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (7,             7,    10,                5,     2500,         1000);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (7,             7,    1401,         700.0),
                                                  (7,             7,    1402,         550.0);

insert into AchievementStep (achievementId, step, achievementPoints, miningQueueReward)
                     values (7,             8,    10,                1);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (7,             8,    1401,         750.0),
                                                  (7,             8,    1402,         600.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward)
                     values (7,             9,    10,                5,     4000);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (7,             9,    1401,         800.0),
                                                  (7,             9,    1402,         650.0);

insert into AchievementStep (achievementId, step, achievementPoints, miningQueueReward)
                     values (7,             10,   10,                1);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (7,             10,   1401,         850.0),
                                                  (7,             10,   1402,         700.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward)
                     values (7,             11,   10,                5,     6000);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (7,             11,   1401,         900.0),
                                                  (7,             11,   1402,         750.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward)
                     values (7,             12,   10,                5,     9999);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (7,             12,   1401,         950.0),
                                                  (7,             12,   1402,         950.0);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (7,             12,   5,     15000);


-- Achievements - Prantum Mastery
insert into Achievement (id, title,             description)
                 values (8,  'Prantum Mastery', 'Master mining in the Prantum areas')
ON DUPLICATE KEY UPDATE title = VALUES(title), description = VALUES(description);

insert into AchievementPredecessor (predecessorId, predecessorStep, successorId)
                            values (7,             5,               8);

insert into AchievementStep (achievementId, step, achievementPoints, miningAreaId, oreId, maxOreReward, maxDepotReward)
                     values (8,             1,    10,                1501,         6,     120,          120);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (8,             1,    5,     1000);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (8,             1,    1402,         450.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (8,             2,    10,                6,     180,          180);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (8,             2,    6,     500);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (8,             3,    10,                6,     250,          250);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (8,             3,    6,     1000);

insert into AchievementStep (achievementId, step, achievementPoints, miningAreaId)
                     values (8,             4,    10,                1502);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (8,             4,    1501,         650.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (8,             5,    10,                6,     600,          600);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (8,             5,    6,     2500);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (8,             5,    1501,         700.0),
                                                  (8,             5,    1502,         650.0);

insert into AchievementStep (achievementId, step, achievementPoints, miningQueueReward)
                     values (8,             6,    10,                1);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (8,             6,    1501,         750.0),
                                                  (8,             6,    1502,         700.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (8,             7,    10,                6,     2500,         1000);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (8,             7,    1501,         800.0),
                                                  (8,             7,    1502,         750.0);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (8,             7,    6,     5000);

insert into AchievementStep (achievementId, step, achievementPoints, miningQueueReward)
                     values (8,             8,    10,                1);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (8,             8,    1501,         850.0),
                                                  (8,             8,    1502,         800.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward)
                     values (8,             9,    10,                6,     4000);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (8,             9,    1501,         900.0),
                                                  (8,             9,    1502,         850.0);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (8,             9,    6,     7500);

insert into AchievementStep (achievementId, step, achievementPoints, miningQueueReward)
                     values (8,             10,   10,                1);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (8,             10,   1502,         900.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward)
                     values (8,             11,   10,                6,     6000);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (8,             11,   1501,         910.0),
                                                  (8,             11,   1502,         910.0);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (8,             11,   6,     10000);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward)
                     values (8,             12,   10,                6,     9999);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (8,             12,   1501,         950.0),
                                                  (8,             12,   1502,         950.0);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (8,             12,   6,     15000);

-- Achievements - Raxia Mastery
insert into Achievement (id, title,             description)
                 values (9,  'Raxia Mastery', 'Master mining in the Raxia areas')
ON DUPLICATE KEY UPDATE title = VALUES(title), description = VALUES(description);

insert into AchievementPredecessor (predecessorId, predecessorStep, successorId)
                            values (8,             5,               9);

insert into AchievementStep (achievementId, step, achievementPoints, miningAreaId, oreId, maxOreReward, maxDepotReward)
                     values (9,             1,    10,                1601,         7,     300,          300);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (9,             1,    6,     3000);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (9,             1,    1502,         550.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (9,             2,    10,                7,     350,          350);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (9,             2,    7,     1000);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (9,             3,    10,                7,     500,          500);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (9,             3,    7,     4000);

insert into AchievementStep (achievementId, step, achievementPoints, miningAreaId)
                     values (9,             4,    10,                1602);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (9,             4,    1601,         650.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (9,             5,    10,                7,     750,          750);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (9,             5,    7,     7500);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (9,             5,    1601,         700.0),
                                                  (9,             5,    1602,         650.0);

insert into AchievementStep (achievementId, step, achievementPoints, miningQueueReward)
                     values (9,             6,    10,                1);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (9,             6,    1601,         750.0),
                                                  (9,             6,    1602,         700.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward, maxDepotReward)
                     values (9,             7,    10,                7,     2500,         1000);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (9,             7,    1601,         800.0),
                                                  (9,             7,    1602,         750.0);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (9,             7,    7,     15000);

insert into AchievementStep (achievementId, step, achievementPoints, miningQueueReward)
                     values (9,             8,    10,                1);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (9,             8,    1601,         850.0),
                                                  (9,             8,    1602,         800.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward)
                     values (9,             9,    10,                7,     4000);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (9,             9,    1601,         900.0),
                                                  (9,             9,    1602,         850.0);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (9,             9,    7,     25000);

insert into AchievementStep (achievementId, step, achievementPoints, miningQueueReward)
                     values (9,             10,   10,                1);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (9,             10,   1602,         900.0);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward)
                     values (9,             11,   10,                7,     6000);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (9,             11,   1601,         910.0),
                                                  (9,             11,   1602,         910.0);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (9,             11,   7,     40000);

insert into AchievementStep (achievementId, step, achievementPoints, oreId, maxOreReward)
                     values (9,             12,   10,                7,     9999);
insert into AchievementStepMiningScoreRequirement (achievementId, step, miningAreaId, minimumScore)
                                           values (9,             12,   1601,         950.0),
                                                  (9,             12,   1602,         950.0);
insert into AchievementStepMiningTotalRequirement (achievementId, step, oreId, amount)
                                           values (9,             12,   7,     75000);



-- Calculate the tier levels
update RobotPart
set tierId = 
(
select max(OrePriceAmount.oreId)
from OrePriceAmount
where OrePriceAmount.orePriceId = RobotPart.orePriceId
);


-- Update depot values
update UserOreAsset
set depotMaxAllowed = COALESCE(
(
 select max(AchievementStep.maxDepotReward)
 from AchievementStep, UserAchievement
 where AchievementStep.achievementId = UserAchievement.achievementId
 and AchievementStep.step <= UserAchievement.stepsClaimed
 and AchievementStep.oreId = UserOreAsset.oreId
 and UserAchievement.userId = UserOreAsset.userId
 and AchievementStep.maxDepotReward is not null
), 0);
