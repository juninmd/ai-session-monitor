import { MockBackend, type MockOptions } from './MockBackend';

export { SCENARIO_NAMES, type ScenarioName, scenarioFromSearch } from './scenarios';
export { MockBackend, type MockOptions };

export function createMockBackend(options: MockOptions = {}): MockBackend {
  return new MockBackend(options);
}
