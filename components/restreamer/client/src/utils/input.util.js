import { isFailoverInput } from './util';
import omit from 'lodash/omit';
import isEqual from 'lodash/isEqual';

export const getEndpointsWithStreamsErrors = (input) => {
  if (isFailoverInput(input)) {
    const endpoints = input.src.inputs
      .map((i) => [i.key, i.endpoints.filter((e) => e.streamStat)[0]])
      .filter((x) => x[1] && x[1].streamStat.error);

    return endpoints.map(x => x[0]);
  }

  return [];
}

export const getEndpointsWithDiffStreams = (input) => {
  if (isFailoverInput(input)) {
    const endpoints = input.src.inputs
      .map((i) => [i.key, i.endpoints.filter((e) => e.streamStat)[0]])
      .filter(([_, streamStat] = x) => streamStat);

    if(!endpoints?.length) {
      return false;
    }

    const excludeProps = ['videoRFrameRate', 'bitRate'];
    const [[firstEndpointKey, { streamStat: firstStreamStat }], _] = endpoints;
    const endpointsWithDiffStreams = endpoints
      .slice(1)
      .reduce((diffKeys, [currentKey, { streamStat: currentStreamStat }] = current) => {
        if (!isEqual(omit(currentStreamStat, excludeProps), omit(firstStreamStat,excludeProps))) {
          diffKeys = [...diffKeys, currentKey];
        }

        return diffKeys;
      }, []);

    return { firstEndpointKey, endpointsWithDiffStreams };
  }

  return false;
}
