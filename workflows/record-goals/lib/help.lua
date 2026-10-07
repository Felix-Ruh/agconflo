-- What the record-goals workflow's scripts share. A script reaches it with
-- `local help = require('help')`.
local help = {}

-- What every model step's prompt begins with: the step offers no tools.
help.NO_TOOLS = "You have no tools in this step. Do not call a tool and do not write out a tool call in any syntax: answer in plain text only, in exactly the form asked for below.\n\n"

-- What a looped model step's instruction ends with: the goal it is given
-- carries the judge's feedback.
help.GOAL_NOTE = " The goal below says which goal of how many this is and which pass. If its FEEDBACK is not 'none', a reviewer found what it says wrong with the last attempt at this goal: correct whatever of it concerns your answer."

-- The lines of `s`, the last one included when it has no newline.
function help.lines(s) local t = {} for l in (s .. "\n"):gmatch("(.-)\n") do t[#t+1] = l end return t end

-- `s` without the white space it begins and ends with.
function help.trim(s) return (s:gsub("^%s+", ""):gsub("%s+$", "")) end

-- A hint naming the goal in `goals`, a table of the graph's goals, whose title
-- shares the most words with the id `id`, or "" when none shares one.
function help.suggest(id, goals)
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

-- The sentences of a directive's body: its lines indented three spaces that
-- are not options, joined, and cut after each full stop, semicolon or colon.
function help.body_sentences(d)
  local text = {}
  for _, l in ipairs(help.lines(d)) do if l:match("^   [^:%s]") then text[#text+1] = help.trim(l) end end
  local s, out = table.concat(text, " "), {}
  for sent in (s .. " "):gmatch("(.-[%.;:])%s") do if help.trim(sent) ~= "" then out[#out+1] = help.trim(sent) end end
  return out
end

-- A list of what failed, `fails`, for a script whose host is `host`:
-- `need(ok, what)` adds `what` unless `ok`; `clean(s, what)` needs `s` to be
-- plain text and not empty; `report(name)` is the script's output, PASS or
-- FAIL with every failure.
function help.checks(host)
  local c = { fails = {} }
  function c.need(ok, what) if not ok then c.fails[#c.fails+1] = what end end
  function c.clean(s, what)
    c.need(not (s:find("DSML", 1, true) or s:find("<invoke", 1, true) or s:find("<\239\189\156", 1, true)),
         what .. " is tool-call markup, not the plain text asked for")
    c.need(help.trim(s) ~= "", what .. " is empty")
  end
  function c.report(name)
    if #c.fails == 0 then return host.text(host.output, "PASS " .. name) end
    return host.text(host.output, "FAIL " .. name .. ": " .. table.concat(c.fails, "; "))
  end
  return c
end

-- The model playing `role` asked `base`, and its answer checked by
-- `check(out, need, clean)`; while a check fails, asked again with its previous
-- answer and the reasons, at most three times in all. The last answer, as the
-- step's output.
function help.ask(host, role, base, check)
  local out, why = nil, nil
  for attempt = 1, 3 do
    local text = base
    if out then text = text .. "\n\n## your previous answer\n\n" .. out .. "\n\n## why it was refused\n\n" .. why .. "\n\nWrite the whole answer again, correcting every point." end
    out = host.complete(role, host.text(host.output, text)):render()
    local c = help.checks(host)
    check(out, c.need, c.clean)
    if #c.fails == 0 then break end
    why = table.concat(c.fails, "\n")
  end
  return host.text(host.output, out)
end

return help
