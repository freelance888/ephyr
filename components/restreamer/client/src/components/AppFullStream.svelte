<script lang="js">
  import { createGraphQlClient, isYoutubeVideo } from '../utils/util';

  import {
    DisableOutput,
    EnableOutput,
    Files,
    Info,
    RemoveOutput,
    ServerInfo,
    SingleRestream,
    TuneDelay,
    TuneVolume,
  } from '../../api/client.graphql';
  import { setClient, subscribe } from 'svelte-apollo';
  import Shell from './common/Shell.svelte';
  import Playlist from './Playlist.svelte';
  import OutputModal from '../modals/OutputModal.svelte';
  import YoutubePlayer from './common/YoutubePlayer.svelte';
  import Restream from './Restream.svelte';

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
  const restreamId = urlParams.get('restream-id');

  let isOnline = false;
  const singleRestream = subscribe(SingleRestream, {
    variables: { id: restreamId.toString() },
    errorPolicy: 'all',
  });
  const info = subscribe(Info, { errorPolicy: 'all' });
  const serverInfo = subscribe(ServerInfo, { errorPolicy: 'all' });
  const filesInfo = subscribe(Files, { errorPolicy: 'all' });

  let title = document.title;
  $: document.title = (isOnline ? '' : '🔴  ') + title;

  $: infoError = $info?.error;
  $: isLoading = !isOnline || $singleRestream.loading;
  $: canRenderMainComponent =
    isOnline && $singleRestream.data && $info.data && $filesInfo?.data;
  $: restreamError = $singleRestream?.error;
  $: sInfo = $serverInfo?.data?.serverInfo;
  $: restream = canRenderMainComponent && $singleRestream?.data?.restream;
  $: filesError = $filesInfo?.error;
  $: files = (canRenderMainComponent && $filesInfo?.data?.files) || [];

  $: translationYoutubeUrl =
    canRenderMainComponent &&
    restream.outputs
      .filter((x) => isYoutubeVideo(x.previewUrl))
      .map((x) => x.previewUrl)[0];

  $: playlist = restream?.playlist;
</script>

<template>
  <Shell
    {isLoading}
    {canRenderMainComponent}
    error={restreamError || infoError | filesError}
    serverInfo={sInfo}
  >
    <div slot="main">
      <OutputModal />
      <div class="section-title">{restream.key}</div>
      <Restream
        public_host={$info.data.info.publicHost}
        value={restream}
        isFullView="true"
        globalOutputsFilters={[]}
      />
      <div class="section-title">Playlist</div>
      <section class="uk-section uk-section-muted uk-padding-remove">
        <Playlist restreamId={restream.id} {playlist} {files} />
      </section>
      {#if translationYoutubeUrl}
        <div class="section-title">Watch translation</div>
        <section class="uk-section uk-section-muted video-player">
          <YoutubePlayer preview_url={translationYoutubeUrl} />
        </section>
      {/if}
    </div>
  </Shell>
</template>

<style lang="stylus">
  .section-title
    margin-top: 8px
    margin-bottom: 4px
    font-size: 1.2rem
    text-transform: uppercase

  .video-player
    padding: 16px
    max-height: 800px
    min-height: 150px

</style>
