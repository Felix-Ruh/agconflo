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

local d, b = given.fdiff:render(), given.brief:render()
need(given.fubc:render():find("No errors found.", 1, true) ~= nil, "ubc check of the whole change reported errors")
for _, s in ipairs(lines(given.statements:render())) do
  if trim(s) ~= "" then need(d:find("+   :statement: " .. trim(s), 1, true) ~= nil, "not recorded: " .. trim(s)) end
end
local ws = lines(given.given_stakeholders:render())
local k = 0
for _, s in ipairs(lines(given.statements:render())) do
  if trim(s) ~= "" then
    k = k + 1
    local w = trim(ws[k] or "unset")
    if w ~= "unset" then need(d:find("+   :stakeholder: " .. w .. "\n+   :statement: " .. trim(s), 1, true) ~= nil, "not recorded with the stakeholder " .. w .. ": " .. trim(s)) end
  end
end
for f in d:gmatch("\n%+%+%+ b/(%S+)") do need(f:match("^docs/stakeholder/") ~= nil, "changed outside docs/stakeholder: " .. f) end
local named, missing = 0, {}
for id in b:gmatch("[A-Z][A-Z0-9]*_[A-Z0-9_]+") do
  named = named + 1
  if not d:find(id, 1, true) then missing[#missing+1] = id end
end
need(#missing == 0, "needs the brief names but no body names: " .. table.concat(missing, ", "))
return report("the whole change: ubc, every statement, only stakeholder files, " .. (named == 0 and "and the brief names no need" or "every need the brief names"))
