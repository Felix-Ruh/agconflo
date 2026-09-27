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

local ks = {}
for _, l in ipairs(lines(given.keywords:render())) do
  local k = trim(l):lower()
  if k:match("^[a-z]+$") then ks[#ks+1] = '"' .. k .. '"' end
end
if #ks == 0 then ks[1] = '"nothing"' end
return host.text(host.output, "cd repo && ubc query cypher --project docs --strict 'MATCH (n:dec) WITH n, size([k IN [" .. table.concat(ks, ", ") .. "] WHERE toLower(n.title) CONTAINS k OR toLower(n.statement) CONTAINS k OR toLower(n.content) CONTAINS k]) AS hits WHERE hits > 0 RETURN n.id, n.title, n.statement, hits ORDER BY hits DESC, n.id LIMIT 20'")
