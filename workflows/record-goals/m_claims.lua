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
  "You have no tools in this step. Do not call a tool and do not write out a tool call in any syntax: answer in plain text only, in exactly the form asked for below.\n\nList every sentence of the draft's body below that states how Agconflo behaves, what it already does, or what another need says, leaving out sentences that name a need by its id (another step checks those), sentences giving the reason the goal exists and sentences saying what the goal deliberately does not say. For each, one line: 'C<n>: \"<the sentence>\" - SUPPORTED BY <ID>', naming a need listed below that says it, or 'C<n>: \"<the sentence>\" - UNSUPPORTED' when none does. A need supports a sentence only if it says the same thing, not merely the same topic. If there is no such sentence, answer exactly NONE.",
  "\n\n## draft\n\n" .. given.draft:render(),
  "\n\n## named\n\n" .. given.named:render(),
  "\n\n## goals\n\n" .. given.goals:render(),
  "\n\n## decs\n\n" .. given.decs:render(),
  "\n\n## current\n\n" .. given.current:render()
})
local out, why = nil, nil
for attempt = 1, 3 do
  local text = base
  if out then text = text .. "\n\n## your previous answer\n\n" .. out .. "\n\n## why it was refused\n\n" .. why .. "\n\nWrite the whole answer again, correcting every point." end
  out = host.complete('reviewing', host.text(host.output, text)):render()
  fails = {}

  clean(out, "the answer")
  local known = given.goals:render() .. "\n" .. given.decs:render() .. "\n" .. given.named:render() .. "\n" .. given.current:render()

  if trim(out) ~= "NONE" then
    local n = 0
    for l in out:gmatch("[^\n]+") do
      if l:match("^%W*C%d+") then
        n = n + 1
        local id = l:match("SUPPORTED BY%W*([A-Z][A-Z0-9_]+)")
        need(id ~= nil or l:find("UNSUPPORTED", 1, true), "a claim without SUPPORTED BY <ID> or UNSUPPORTED: " .. l:sub(1, 60))
        if id then need(known:find(id, 1, true) ~= nil, "no such need: " .. id) end
      end
    end
    need(n >= 1, "no line 'C<n>: ...', and not NONE")
  end

  if #fails == 0 then break end
  why = table.concat(fails, "\n")
end
return host.text(host.output, out)
