"""Trace a run of a workflow from its record: each instance's passes, what its
model was shown and answered, the tools it called, each output and each route.

    python3 workflows/record-goals/trace.py <run.toml> <graph.json> <trace.json>

Writes the trace as JSON and prints one line per instance.
"""
import collections, json, sys, tomllib

record, graph, out = sys.argv[1], sys.argv[2], sys.argv[3]
r = tomllib.load(open(record, 'rb'))
contexts = r['context']


def text(cid):
    c = contexts.get(str(cid))
    if c is None:
        return ''
    if 'text' in c:
        return c['text']
    return c.get('separator', '').join(text(p) for p in c.get('parts', []))


nodes = {n['name']: n for n in json.load(open(graph))}
instances = collections.OrderedDict()
pending = {}
timeline = []


def of(name):
    return instances.setdefault(name, {'passes': [], 'cur': None})


def current(d):
    if d['cur'] is None:
        d['cur'] = {'turns': [], 'tools': collections.Counter()}
    return d['cur']


for e in r['event']:
    d = of(e['instance'])
    if 'exchange' in e:
        x = e['exchange']
        current(d)['turns'].append({'window': len(text(x['window'])), 'shown': text(x['window']),
                                    'answer': text(x['answer']), 'calls': []})
    elif 'call' in e:
        k = e['call']
        c = current(d)
        c['tools'][k['node_type']] += 1
        call = {'tool': k['node_type'], 'input': ' | '.join(text(v) for v in k['inputs'].values())}
        if c['turns']:
            c['turns'][-1]['calls'].append(call)
        pending[k['id']] = call
    elif 'performing' in e:
        call = pending.get(e['performing'])
        if call:
            call['output'] = text(e['output'])
    elif 'output' in e:
        c = current(d)
        c['output'] = text(e['output'])
        c['route'] = e.get('route')
        c['tools'] = dict(c['tools'])
        d['passes'].append(c)
        d['cur'] = None
        timeline.append((e['instance'], len(d['passes']), e.get('route')))

result = {
    'budget': r['budget'], 'spent': r['spent'], 'timeline': timeline,
    'instances': [dict(name=n, kind=nodes.get(n, {}).get('kind'), stage=nodes.get(n, {}).get('stage'),
                       passes=instances[n]['passes'], outstanding=instances[n]['cur'] is not None)
                  for n in instances],
}
json.dump(result, open(out, 'w'), indent=1)
print('spent %s of %s' % (r['spent'], r['budget']))
for i in result['instances']:
    ps = i['passes']
    last = (ps[-1]['output'][:70] if ps else '').replace('\n', ' / ')
    print('%-22s %-7s passes %-3d last %r' % (i['name'], i['kind'] or '', len(ps), last))
