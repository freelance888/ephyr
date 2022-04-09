<script lang="js">
  import { createGraphQlClient } from '../utils/util';

  import { Files, Info, ServerInfo, State } from '../../api/client.graphql';
  import { setClient, subscribe } from 'svelte-apollo';
  import Shell from './common/Shell.svelte';
  import Restream from './Restream.svelte';

  const gqlClient = createGraphQlClient(
    '/api',
    () => (isOnline = true),
    () => (isOnline = false)
  );
  setClient(gqlClient);

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

  const urlParams = new URLSearchParams(window.location.search);
  const restream_id = urlParams.get('id');

  function getRestream() {
    return canRenderMainComponent && $state.data.allRestreams.find(x => x.id === restream_id);
  }

</script>

<template>
  <Shell
    {isLoading}
    {canRenderMainComponent}
    error={stateError || infoError}
    serverInfo={sInfo}
  >
  <div slot="main" >
      <Restream
        public_host={$info.data.info.publicHost}
        value={getRestream()}
        {files}
        isFullView="true"
        globalOutputsFilters={[]}
      />
  </div>
  </Shell>
</template>
