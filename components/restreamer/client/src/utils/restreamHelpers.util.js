import { ONLINE, OFFLINE, INITIALIZING, UNSTABLE } from '../constants/statuses';

export const getReStreamOutputsCount = (reStream) =>
  reStream.outputs.reduce(
    (acc, output) => {
      const outputStatus = output.status;

      acc[outputStatus]++;

      return acc;
    },
    {
      [OFFLINE]: 0,
      [INITIALIZING]: 0,
      [ONLINE]: 0,
      [UNSTABLE]: 0,
    }
  );