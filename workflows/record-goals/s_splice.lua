local given, host = ...

local function lines(s) local t = {} for l in (s .. "\n"):gmatch("(.-)\n") do t[#t+1] = l end return t end
local function trim(s) return (s:gsub("^%s+", ""):gsub("%s+$", "")) end
local fails = {}
local function need(ok, what) if not ok then fails[#fails+1] = what end end
local function clean(s, what)
  need(not (s:find("DSML", 1, true) or s:find("<invoke", 1, true) or s:find("<\239\189\156", 1, true)),
       what .. " is tool-call markup, not the plain text asked for")
  need(trim(s) ~= "", what .. " is empty")
end
local function suggest(id, goals)
  local words, best, bid, btitle = {}, 0, nil, nil
  for w in id:lower():gmatch("[a-z]+") do if #w > 2 and w ~= "stkh" then words[#words+1] = w end end
  for gid, title in (goals or ""):gmatch("\n(STKH_[A-Z0-9_]+)%s+(.-)%s%s+") do
    local score, t = 0, title:lower()
    for _, w in ipairs(words) do if t:find(w, 1, true) then score = score + 1 end end
    if score > best then best, bid, btitle = score, gid, title end
  end
  if bid then return " (did you mean " .. bid .. ', "' .. btitle .. '"?)' end
  return ""
end
local function report(name)
  if #fails == 0 then return host.text(host.output, "PASS " .. name) end
  return host.text(host.output, "FAIL " .. name .. ": " .. table.concat(fails, "; "))
end

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
