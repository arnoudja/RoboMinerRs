-- Migration 011: worker processing lease so multiple rally engines cannot
-- select the same unfinished MiningQueue rows while a simulation is in flight.

ALTER TABLE MiningQueue
    ADD COLUMN processingLeaseUntil TIMESTAMP NULL;
