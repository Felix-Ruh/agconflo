local given, host = ...
local help = require('help')
local lines, trim, suggest = help.lines, help.trim, help.suggest
local base = table.concat({
  help.NO_TOOLS .. "From the needs below, list the existing needs the goal's body should name: any the brief names that concern this goal, those it depends on, those it could be mistaken for, and decisions that already make room for it. Give 2 to 4 lines, one per need, first any need the brief names that concerns this goal, each as the id, a colon, and one sentence copied exactly from that need's statement as it appears below. Use only ids that appear below." .. help.GOAL_NOTE,
  "\n\n## goal\n\n" .. given.goal:render(),
  "\n\n## brief\n\n" .. given.brief:render(),
  "\n\n## named\n\n" .. given.named:render(),
  "\n\n## goals\n\n" .. given.goals:render(),
  "\n\n## decs\n\n" .. given.decs:render(),
  "\n\n## current\n\n" .. given.current:render()
})
return help.ask(host, 'drafting', base, function(out, need, clean)
  clean(out, "the answer")
  local known = given.goals:render() .. "\n" .. given.decs:render() .. "\n" .. given.named:render() .. "\n" .. given.current:render()
  local n = 0
  for _, l in ipairs(lines(trim(out))) do
    local id, q = l:match("^%W*([A-Z][A-Z0-9]*_[A-Z0-9_]+)%W*:%s*(.-)%s*$")
    if id then
      n = n + 1
      need(known:find(id, 1, true) ~= nil, "no such need: " .. id .. suggest(id, given.goals:render()))
      q = q:gsub('^"', ""):gsub('"$', "")
      need(#q > 10 and known:find(q:sub(1, 60), 1, true) ~= nil, "the sentence given for " .. id .. " is not copied exactly from what is listed")
    elseif trim(l) ~= "" then need(false, "a line is not 'ID: sentence': " .. l:sub(1, 60)) end
  end
  need(n >= 2 and n <= 4, "expected 2 to 4 needs, got " .. n)
end)
