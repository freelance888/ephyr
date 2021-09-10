import { ONLINE, OFFLINE, INITIALIZING } from '../constants/statuses';

export const getAggregatedStreamsData = (streams) =>
  streams.reduce(
    (acc, stream) => {
      const streamInputStatus = stream.input.endpoints[0].status;

      stream.outputs.forEach((output) => {
        acc.outputsCountByStatus[output.status]++;
      });

      acc.inputsCountByStatus[streamInputStatus]++;

      return acc;
    },
    {
      inputsCountByStatus: {
        [OFFLINE]: 0,
        [INITIALIZING]: 0,
        [ONLINE]: 0,
      },
      outputsCountByStatus: {
        [OFFLINE]: 0,
        [INITIALIZING]: 0,
        [ONLINE]: 0,
      },
    }
  );
