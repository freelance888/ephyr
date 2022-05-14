<script lang='js'>
  import { createGraphQlClient } from '../utils/util';

  import {
    Files,
    Info,
    ServerInfo,
    State,
    TuneVolume,
    TuneDelay,
    DisableOutput,
    EnableOutput, RemoveOutput
  } from '../../api/client.graphql';
  import { setClient, subscribe } from 'svelte-apollo';
  import Shell from './common/Shell.svelte';
  import Restream from './Restream.svelte';
  import Playlist from './Playlist.svelte';
  import Output from './Output.svelte';

  let outputMutations = {
    DisableOutput,
    EnableOutput,
    RemoveOutput,
    TuneVolume,
    TuneDelay,
  };

  const gqlClient = createGraphQlClient(
    '/api',
    () => (isOnline = true),
    () => (isOnline = false)
  );
  setClient(gqlClient);

  const urlParams = new URLSearchParams(window.location.search);
  const translationRestreamId = urlParams.get('tran_restream_id');
  const parentOutputId = urlParams.get('parent_output_id');

  let isOnline = false;
  const info = subscribe(Info, { errorPolicy: 'all' });
  const state = subscribe(State, { errorPolicy: 'all' });
  const serverInfo = subscribe(ServerInfo, { errorPolicy: 'all' });
  const files = subscribe(Files, { errorPolicy: 'all' });

  let title = document.title;
  $: document.title = (isOnline ? '' : '🔴  ') + title;

  $: infoError = $info && $info.error;
  $: isLoading = !isOnline || $state.loading;
  $: canRenderMainComponent = isOnline && $state.data && $info.data;
  $: stateError = $state && $state.error;
  $: sInfo = $serverInfo && $serverInfo.data && $serverInfo.data.serverInfo;

  $: translationRestream = canRenderMainComponent && $state.data.allRestreams.find(x => x.id === translationRestreamId)
  $: parentRestreamWithMixOutput = canRenderMainComponent && $state.data.allRestreams
    .reduce((acc, x) => acc.concat(x.outputs.map(o => ({restreamId: x.id, output: o}))), [])
    .find(x => x.output.id === parentOutputId)

</script>

<template>
  <Shell
    {isLoading}
    {canRenderMainComponent}
    error={stateError || infoError}
    serverInfo={sInfo}
  >
    <div slot='main'>
      <div class='section-title'>{translationRestream.key}</div>
      <Restream
        public_host={$info.data.info.publicHost}
        value={translationRestream}
        {files}
        isFullView='true'
        globalOutputsFilters={[]}
      />
      <div class='section-title'>Sound mixer</div>
      <section class='uk-section uk-section-muted custom-section single-output'>
        <Output restream_id={parentRestreamWithMixOutput.restreamId} value={parentRestreamWithMixOutput.output} mutations={outputMutations} />
      </section>
      <div class='section-title'>Playlist</div>
      <section class='uk-section uk-section-muted custom-section'>
        <Playlist />
      </section>

    </div>

  </Shell>
</template>
<style lang='stylus'>
  .section-title
    margin-top: 8px
    margin-bottom: 4px
    font-size: 1.2rem
    text-transform: uppercase

  .custom-section
    padding: 10px 20px 20px 0

  .single-output
    :global(.volume input)
      width: 90% !important
</style>
