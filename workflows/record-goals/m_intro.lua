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
  "You have no tools in this step. Do not call a tool and do not write out a tool call in any syntax: answer in plain text only, in exactly the form asked for below.\n\nBelow is the introduction of the file the goal goes in. It names the themes the file's goals cover, as short lowercase clauses. If the goal below adds a theme the introduction does not name yet, write the introduction again with that theme added as one more clause in the same style, changing nothing else, wrapped at 79 columns. Otherwise answer exactly UNCHANGED. Answer with the paragraph or UNCHANGED and nothing else - no heading, no quotes, no commentary. The goal below says which goal of how many this is and which pass. If its FEEDBACK is not 'none', a reviewer found what it says wrong with the last attempt at this goal: correct whatever of it concerns your answer.",
  "\n\n## goal\n\n" .. given.goal:render(),
  "\n\n## introduction\n\n" .. given.introduction:render(),
  "\n\n## file_goals\n\n" .. given.file_goals:render()
})
local out, why = nil, nil
for attempt = 1, 3 do
  local text = base
  if out then text = text .. "\n\n## your previous answer\n\n" .. out .. "\n\n## why it was refused\n\n" .. why .. "\n\nWrite the whole answer again, correcting every point." end
  out = host.complete('drafting', host.text(host.output, text)):render()
  fails = {}

  clean(out, "the introduction")
  if trim(out) ~= "UNCHANGED" then
    local n, first = 0, lines(given.introduction:render())[1] or ""
    for _, l in ipairs(lines(trim(out))) do
      n = n + 1
      need(#l <= 79, "line " .. n .. " is longer than 79 columns")
      need(not l:match("^%.%. ") and not l:match("^   :"), "line " .. n .. " is not prose")
    end
    need(n <= 10, "it is longer than ten lines")
    local s = out:gsub("``[^`]*``", "")
    need(not s:find(", [A-Z]"), "a clause after a comma starts with a capital letter: every theme is a lowercase clause like the others")
    need(trim(out):sub(1, 30) == first:sub(1, 30), "it does not begin as the introduction does: " .. first:sub(1, 30))
  end

  if #fails == 0 then break end
  why = table.concat(fails, "\n")
end
return host.text(host.output, out)
