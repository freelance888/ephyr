<script lang="js">
  import { createGraphQlClient, isYoutubeVideo } from '../utils/util';

  import {
    DisableOutput,
    EnableOutput,
    Files,
    Info,
    RemoveOutput,
    RestreamWithParent,
    ServerInfo,
    TuneDelay,
    TuneSidechain,
    TuneVolume,
  } from '../../api/client.graphql';
  import { setClient, subscribe } from 'svelte-apollo';
  import Shell from './common/Shell.svelte';
  import Playlist from './Playlist.svelte';
  import OutputModal from '../modals/OutputModal.svelte';
  import YoutubePlayer from './common/YoutubePlayer.svelte';
  import Restream from './Restream.svelte';
  import Output from './Output.svelte';
  import {
    FILE_DOWNLOAD_ERROR,
    FILE_LOCAL,
    isDownloadingState,
  } from '../utils/constants';
  import StreamInfoDiffTooltip from './common/StreamInfoDiffTooltip.svelte';
  import { getPlaylistItemsWithDiffStreams } from '../utils/streamInfo.util';

  let outputMutations = {
    DisableOutput,
    EnableOutput,
    RemoveOutput,
    TuneVolume,
    TuneDelay,
    TuneSidechain,
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
  const restreamWithParent = subscribe(RestreamWithParent, {
    variables: { id: restreamId.toString() },
    errorPolicy: 'all',
  });
  const info = subscribe(Info, { errorPolicy: 'all' });
  const serverInfo = subscribe(ServerInfo, { errorPolicy: 'all' });
  const filesInfo = subscribe(Files, { errorPolicy: 'all' });

  let title = document.title;
  $: document.title = (isOnline ? '' : '🔴  ') + title;

  $: infoError = $info?.error;
  $: isLoading = !isOnline || $restreamWithParent.loading;
  $: canRenderMainComponent =
    isOnline && $restreamWithParent?.data && $info?.data && $filesInfo?.data;

  $: restreamError = $restreamWithParent?.error;
  $: sInfo = $serverInfo?.data?.serverInfo;
  $: filesError = $filesInfo?.error;
  $: files = (canRenderMainComponent && $filesInfo?.data?.files) || [];

  $: restream =
    canRenderMainComponent &&
    $restreamWithParent.data?.restreamWithParent?.restream;
  $: parentData =
    canRenderMainComponent &&
    $restreamWithParent.data?.restreamWithParent?.parent;
  $: parentRestreamOutput =
    canRenderMainComponent &&
    parentData?.restream?.outputs?.find((o) => o.id === parentData.outputId);

  $: translationYoutubeUrl =
    canRenderMainComponent &&
    restream?.outputs
      .filter((x) => isYoutubeVideo(x.previewUrl))
      .map((x) => x.previewUrl)[0];

  $: playlist = restream?.playlist;

  $: playlistQueue = playlist
    ? playlist.queue
        .map((x) => ({
          id: x.fileId,
          name: x.name ?? x.fileId,
          isPlaying: playlist.currentlyPlayingFile
            ? playlist.currentlyPlayingFile.fileId === x.fileId
            : false,
          file: files.find((f) => f.fileId === x.fileId),
          wasPlayed: x.wasPlayed,
        }))
        .map((x) => ({
          ...x,
          isLocal: x.file?.state === FILE_LOCAL,
          isDownloading: isDownloadingState(x.file?.state),
          isError: x.file?.state === FILE_DOWNLOAD_ERROR,
        }))
    : [];

  $: currentlyPlayingFileId = playlist?.currentlyPlayingFile?.fileId;

  $: streamsErrorsTooltip = getStreamErrorTooltip(playlistQueue);

  $: streamsDiffTooltip = getStreamsDifferenceTooltip(playlistQueue);

  const getStreamsDifferenceTooltip = (queue) => {
    const result = getPlaylistItemsWithDiffStreams(queue);

    return Array.isArray(result)
      ? `There are streams with different streams params&colon; <br>
<strong>${result.join('<br>')}</strong>`
      : '';
  };

  const getStreamErrorTooltip = (queue) => {
    const filesNames = queue.filter((x) => x.file?.error).map((x) => x.name);
    return filesNames?.length
      ? `Can't get stream info from&colon; <br><strong>${filesNames.reduce(
          (acc, cur) => (acc += '<br>' + cur)
        )}</strong>`
      : '';
  };
</script>

<template>
  <Shell
    {isLoading}
    {canRenderMainComponent}
    error={restreamError || infoError || filesError}
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
      {#if parentRestreamOutput && parentRestreamOutput.mixins?.length > 0}
        <div class="section-title">Sound mixer</div>
        <section class="uk-section uk-section-muted single-output">
          <Output
            restream_id={parentData.restream?.id}
            value={parentRestreamOutput}
            isReadOnly="true"
            mutations={outputMutations}
          />
        </section>
      {/if}
      <div>
        <span class="section-title">Playlist</span>
        {#if streamsErrorsTooltip}
          <StreamInfoDiffTooltip {streamsErrorsTooltip} />
        {/if}
        {#if streamsDiffTooltip}
          <StreamInfoDiffTooltip {streamsDiffTooltip} />
        {/if}
      </div>
      <section class="uk-section uk-section-muted uk-padding-remove">
        <Playlist
          restreamId={restream.id}
          queue={playlistQueue}
          {currentlyPlayingFileId}
        />
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

  .single-output
    padding: 16px
    :global(.volume input)
      width: 90% !important

  .video-player
    padding: 16px
    max-height: 800px
    min-height: 150px

</style>
