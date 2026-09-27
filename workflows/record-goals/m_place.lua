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
  "You have no tools in this step. Do not call a tool and do not write out a tool call in any syntax: answer in plain text only, in exactly the form asked for below.\n\nSay after which goal of the file the goal below goes: beside the goals it is most closely related to. Answer with exactly one line 'AFTER: ' followed by an id from the file's goals listed below, in their order, and nothing else. The goal below says which goal of how many this is and which pass. If its FEEDBACK is not 'none', a reviewer found what it says wrong with the last attempt at this goal: correct whatever of it concerns your answer.",
  "\n\n## goal\n\n" .. given.goal:render(),
  "\n\n## file_goals\n\n" .. given.file_goals:render(),
  "\n\n## neighbours\n\n" .. given.neighbours:render()
})
local out, why = nil, nil
for attempt = 1, 3 do
  local text = base
  if out then text = text .. "\n\n## your previous answer\n\n" .. out .. "\n\n## why it was refused\n\n" .. why .. "\n\nWrite the whole answer again, correcting every point." end
  out = host.complete('drafting', host.text(host.output, text)):render()
  fails = {}

  clean(out, "the answer")
  local a = out:match("AFTER:%s*(STKH_[A-Z0-9_]+)")
  need(a ~= nil, "no line 'AFTER: <id>'")
  if a then need(("\n" .. given.file_goals:render()):find("\n" .. a .. " ", 1, true) ~= nil, a .. " is not a goal in the file") end

  if #fails == 0 then break end
  why = table.concat(fails, "\n")
end
return host.text(host.output, out)
