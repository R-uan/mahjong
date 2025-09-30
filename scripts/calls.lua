-- Bamboos: { 11, 12, 13, 14, 15, 16, 17, 18, 19 }
-- Circles: { 21, 22, 23, 24, 25, 26, 27, 28, 29 }
-- Characters: { 31, 32, 33, 34, 35, 36, 37, 38, 39 }
-- Honors: { (E)41, (S)42, (W)43, (N)44, (WD)45, (GD)46, (RD)47 } 

local function count_tiles(hand)
  local counts = {}
  for _, t in ipairs(hand) do
    counts[t] = (counts[t] or 0) + 1
  end
end

local function is_suit(tile)
  return tile < 40
end
