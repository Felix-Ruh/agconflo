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
  "You have no tools in this step. Do not call a tool and do not write out a tool call in any syntax: answer in plain text only, in exactly the form asked for below.\n\nWrite one stkh_req directive for the goal below and nothing else: no fences, no commentary. Follow the example goal's form exactly: '.. stkh_req:: <title>', then the options indented three spaces (:id:, :stakeholder:, :statement: with the goal's statement copied exactly), a blank line, and a body indented three spaces, wrapped at 79 columns, in three paragraphs: why the goal exists; what it deliberately does not say; and how it stands beside each need in the neighbours list, naming each by its id in double backquotes and saying for each in a sentence or two how the two differ or depend - what either could hold without the other - rather than listing them. Use exactly the id, title and stakeholder the naming gives. Name no need that is not in the neighbours list. Follow the rules below. The goal below says which goal of how many this is and which pass. If its FEEDBACK is not 'none', a reviewer found what it says wrong with the last attempt at this goal: correct whatever of it concerns your answer.",
  "\n\n## goal\n\n" .. given.goal:render(),
  "\n\n## naming\n\n" .. given.naming:render(),
  "\n\n## neighbours\n\n" .. given.neighbours:render(),
  "\n\n## example\n\n" .. given.example:render(),
  "\n\n## rules\n\n" .. given.rules:render(),
  "\n\n## goals\n\n" .. given.goals:render(),
  "\n\n## decs\n\n" .. given.decs:render(),
  "\n\n## current\n\n" .. given.current:render()
})
local out, why = nil, nil
for attempt = 1, 3 do
  local text = base
  if out then text = text .. "\n\n## your previous answer\n\n" .. out .. "\n\n## why it was refused\n\n" .. why .. "\n\nWrite the whole answer again, correcting every point." end
  out = host.complete('drafting', host.text(host.output, text)):render()
  fails = {}
  local g = given.goal:render()
local statement = g:match("STATEMENT: ([^\n]*)") or ""

  local d = out
  clean(d, "the draft")
  local ls = lines(trim(d))
  local nm = given.naming:render()
  local want_id = nm:match("ID:%s*(STKH_[A-Z0-9_]+)") or ""
  local want_title = trim(nm:match("TITLE:%s*([^\n]+)") or "")
  local want_who = nm:match("STAKEHOLDER:%s*(%a+)") or ""
  need(ls[1] == ".. stkh_req:: " .. want_title, "the first line is not '.. stkh_req:: " .. want_title .. "'")
  need(d:find("\n   :id: " .. want_id .. "\n", 1, true) ~= nil, "the id is not ':id: " .. want_id .. "'")
  need(d:find("\n   :stakeholder: " .. want_who .. "\n", 1, true) ~= nil, "the stakeholder is not ':stakeholder: " .. want_who .. "'")
  need(d:match("\n   :statement: ([^\n]*)") == statement, "the statement is not exactly: " .. statement)
  local body = 0
  for i, l in ipairs(ls) do
    if l ~= "" and not l:match("^   ") and i > 1 then need(false, "line " .. i .. " is not indented three spaces") break end
    if #l > 80 and not l:match("^   :statement:") then need(false, "line " .. i .. " is longer than 80 columns") break end
    if l:match("^   [^:]") then body = body + 1 end
  end
  need(body >= 6, "the body is shorter than six lines")
  for id in d:gmatch("[^`]`([A-Z][A-Z0-9]*_[A-Z0-9_]+)`[^`]") do need(false, id .. " is in single backquotes: write ``" .. id .. "``") end
  local nb = given.neighbours:render()
  local seen = {}
  for id in d:gmatch("[A-Z][A-Z0-9]*_[A-Z0-9_]+") do
    if id ~= want_id and not seen[id] then
      seen[id] = true
      need(nb:find(id, 1, true) ~= nil, id .. " is not in the neighbours list" .. suggest(id, given.goals:render()))
    end
  end
  for id in nb:gmatch("([A-Z][A-Z0-9]*_[A-Z0-9_]+)%W*:") do need(seen[id], "the body does not name " .. id .. ", which the neighbours list gives") end

  if #fails == 0 then break end
  why = table.concat(fails, "\n")
end
return host.text(host.output, out)
