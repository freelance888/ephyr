import { ONLINE, OFFLINE, INITIALIZING, UNSTABLE, STREAMS_ERROR, STREAMS_WARNING } from './constants';

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
        [UNSTABLE]: 0,
      },
      endpointsStreamsStatus: {
        [STREAMS_ERROR]: 0,
        [STREAMS_WARNING]: 0,
      },
      outputsCountByStatus: {
        [OFFLINE]: 0,
        [INITIALIZING]: 0,
        [ONLINE]: 0,
        [UNSTABLE]: 0,
      },
    }
  );

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

export const toggleFilterStatus = (filters, filter) => {
  const filterIndex = filters.indexOf(filter);

  return filterIndex === -1
    ? [...filters, filter]
    : filters.filter((item) => item !== filter);
};
