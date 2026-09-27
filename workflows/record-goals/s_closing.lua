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

local function body_sentences(d)
  local text = {}
  for _, l in ipairs(lines(d)) do if l:match("^   [^:%s]") then text[#text+1] = trim(l) end end
  local s, out = table.concat(text, " "), {}
  for sent in (s .. " "):gmatch("(.-[%.;:])%s") do if trim(sent) ~= "" then out[#out+1] = trim(sent) end end
  return out
end

local out, k = {}, 0
for _, sent in ipairs(body_sentences(given.draft:render())) do
  local low = " " .. sent:lower():gsub("[^%a ]", " ") .. " "
  for _, w in ipairs({ " only ", " each ", " every ", " exactly ", " no other " }) do
    if low:find(w, 1, true) then k = k + 1; out[#out+1] = "W" .. k .. ': "' .. sent .. '" uses "' .. trim(w) .. '"'; break end
  end
end
if #out == 0 then return host.text(host.output, "none") end
return host.text(host.output, table.concat(out, "\n"))
