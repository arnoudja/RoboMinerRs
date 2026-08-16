-- Migration 008: split mining-area tax into container vs depot rates.
-- taxRate stays as the container rate (same values as before).
-- depotTaxRate starts at half of that, rounded down.
-- MiningOreResult.depotAmount lets claim tax the two piles separately.

ALTER TABLE MiningArea
    ADD COLUMN depotTaxRate INT NOT NULL DEFAULT 0;

UPDATE MiningArea
    SET depotTaxRate = FLOOR(taxRate / 2);

ALTER TABLE MiningOreResult
    ADD COLUMN depotAmount INT NOT NULL DEFAULT 0;
