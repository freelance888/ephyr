import { isFailoverInput } from './util';
import omit from 'lodash/omit';
import isEqual from 'lodash/isEqual';

export const hasEndpointsWithStreamsErrors = (input) => {
  return !!getEndpointsWithStreamsErrors(input)?.length;
};

export const getEndpointsWithStreamsErrors = (input) => {
  if (isFailoverInput(input)) {
    const endpoints = input.src.inputs
      .map((i) => [i.key, i.endpoints.filter((e) => e.streamStat)[0]])
      .filter((x) => x[1] && x[1].streamStat.error);

    return endpoints.map((x) => x[0]);
  }

  return [];
};

export const hasEndpointsWithDiffStreams = (input) => {
  return !!getEndpointsWithDiffStreams(input)?.endpointsWithDiffStreams?.length;
};

export const getEndpointsWithDiffStreams = (input) => {
  if (isFailoverInput(input)) {
    const endpoints = input.src.inputs
      .map((i) => [i.key, i.endpoints.filter((e) => e.streamStat)[0]])
      .filter(([_, streamStat] = x) => streamStat);

    if (!endpoints?.length) {
      return false;
    }

    const excludedProps = ['videoRFrameRate', 'bitRate'];
    const [[firstEndpointKey, { streamStat: firstStreamStat }], _] = endpoints;
    const endpointsWithDiffStreams = endpoints
      .slice(1)
      .reduce(
        (
          diffKeys,
          [currentKey, { streamStat: currentStreamStat }] = current
        ) => {
          if (
            !isEqual(
              omit(currentStreamStat, excludedProps),
              omit(firstStreamStat, excludedProps)
            )
          ) {
            diffKeys = [...diffKeys, currentKey];
          }

          return diffKeys;
        },
        []
      );

    return { firstEndpointKey, endpointsWithDiffStreams };
  }

  return false;
};
