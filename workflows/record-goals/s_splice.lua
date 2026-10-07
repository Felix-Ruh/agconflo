local given, host = ...
local help = require('help')
local lines, trim = help.lines, help.trim
local ls = lines(given.current:render())
table.remove(ls, 1)
while #ls > 0 and ls[#ls] == "" do table.remove(ls) end
local a = given.place:render():match("AFTER:%s*(STKH_[A-Z0-9_]+)")
local d = lines(trim(given.draft:render()))
local at
for i, l in ipairs(ls) do if l == "   :id: " .. (a or "") then at = i end end
if at then
  local stop = #ls + 1
  for i = at + 1, #ls do if ls[i]:match("^%.%. ") then stop = i break end end
  local out = {}
  for i = 1, stop - 1 do out[#out+1] = ls[i] end
  while #out > 0 and out[#out] == "" do table.remove(out) end
  out[#out+1] = ""
  for _, l in ipairs(d) do out[#out+1] = l end
  out[#out+1] = ""
  for i = stop, #ls do out[#out+1] = ls[i] end
  ls = out
end
local intro = trim(given.intro:render())
if intro ~= "UNCHANGED" and intro ~= "" then
  local st, en
  for i = 4, #ls do
    if not st and trim(ls[i]) ~= "" and not ls[i]:match("^=+$") then st = i end
    if st and trim(ls[i]) == "" then en = i - 1 break end
  end
  if st and en and not ls[st]:match("^%.%. ") then
    local out = {}
    for i = 1, st - 1 do out[#out+1] = ls[i] end
    for _, l in ipairs(lines(intro)) do out[#out+1] = l end
    for i = en + 1, #ls do out[#out+1] = ls[i] end
    ls = out
  end
end
while #ls > 0 and ls[#ls] == "" do table.remove(ls) end
local text = (table.concat(ls, "\n") .. "\n"):gsub("\n\n\n+", "\n\n")
return host.text(host.output, text)
