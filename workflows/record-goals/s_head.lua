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

local st = "\n" .. given.state:render()
local k = tonumber(st:match("\ngoal (%d+)\n") or "1")
local p = tonumber(st:match("\npass (%d+)\n") or "1")
local stage = st:match("\nstage (%a+)\n") or "no"
local fb = st:match("feedback:\n(.*)$") or ""
if trim(fb) == "" then fb = "none" end
local ss = {}
for _, l in ipairs(lines(given.statements:render())) do if trim(l) ~= "" then ss[#ss+1] = trim(l) end end
local ws = lines(given.given_stakeholders:render())
return host.text(host.output, "GOAL " .. k .. " OF " .. #ss .. "\nPASS " .. p .. "\nSTAGE " .. stage ..
  "\nSTATEMENT: " .. (ss[k] or "") .. "\nSTAKEHOLDER GIVEN: " .. trim(ws[k] or "unset") .. "\nFEEDBACK:\n" .. fb)
