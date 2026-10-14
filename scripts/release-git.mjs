import { execFileSync } from 'node:child_process';

export function run(command, args, cwd) {
  return execFileSync(command, args, {
    encoding: 'utf8',
    cwd,
    stdio: ['ignore', 'pipe', 'pipe'],
  }).trim();
}

export function workflowsTree(rev, cwd) {
  return run('git', ['ls-tree', rev, '.github/workflows'], cwd).split(/\s+/)[2] ?? '';
}

export function readCommits(shas, tipRev, cwd) {
  const tipWorkflows = workflowsTree(tipRev, cwd);
  return shas.map((sha) => ({
    sha,
    message: run('git', ['log', '-1', '--format=%B', sha], cwd),
    taggable: tipWorkflows === '' || workflowsTree(sha, cwd) === tipWorkflows,
  }));
}
