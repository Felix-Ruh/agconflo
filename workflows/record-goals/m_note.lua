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
local base = table.concat({
  "You have no tools in this step. Do not call a tool and do not write out a tool call in any syntax: answer in plain text only, in exactly the form asked for below.\n\nBelow is the task, which may carry a maintainer's note, and a draft for one of its goals. List each sentence of the maintainer's note that concerns this goal, and say whether the draft keeps it: one line each, 'N<k>: \"<the note's sentence, copied exactly>\" - KEPT', or '- MISSING' when the draft does not say it, or '- CONTRADICTED' when the draft says the opposite, then ' - ' and the draft's words that show it. If the task has no maintainer's note, or none of its sentences concerns this goal, answer exactly NONE.",
  "\n\n## brief\n\n" .. given.brief:render(),
  "\n\n## goal\n\n" .. given.goal:render(),
  "\n\n## draft\n\n" .. given.draft:render()
})
local out, why = nil, nil
for attempt = 1, 3 do
  local text = base
  if out then text = text .. "\n\n## your previous answer\n\n" .. out .. "\n\n## why it was refused\n\n" .. why .. "\n\nWrite the whole answer again, correcting every point." end
  out = host.complete('reviewing', host.text(host.output, text)):render()
  fails = {}

  clean(out, "the answer")
  if trim(out) ~= "NONE" then
    local b, n = given.brief:render(), 0
    for l in out:gmatch("[^\n]+") do
      if l:match("^%W*N%d+") then
        n = n + 1
        local q = l:match('"(.-)"')
        need(q ~= nil and #q > 12 and b:find(q:sub(1, 50), 1, true) ~= nil, "a note sentence not copied exactly from the task: " .. l:sub(1, 60))
        need(l:find("KEPT", 1, true) or l:find("MISSING", 1, true) or l:find("CONTRADICTED", 1, true), "a line without KEPT, MISSING or CONTRADICTED: " .. l:sub(1, 60))
      end
    end
    need(n >= 1, "no line 'N<k>: ...', and not NONE")
  end

  if #fails == 0 then break end
  why = table.concat(fails, "\n")
end
return host.text(host.output, out)
