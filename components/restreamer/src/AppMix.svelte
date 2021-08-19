<script lang="js">
  import { createGraphQlClient } from './util';
  import { setClient, subscribe } from 'svelte-apollo';
  import Shell from './Shell.svelte';
  import Output from './Output.svelte';
  import {
    Output as Mix,
    TuneVolume,
    TuneDelay,
  } from './api/graphql/mix.graphql';
  import YoutubePlayer from './YoutubePlayer.svelte';
  import { isYoutubeVideo } from './util';

  const mutations = { TuneVolume, TuneDelay };

  const gqlClient = createGraphQlClient(
    '/api-mix',
    () => (isOnline = true),
    () => (isOnline = false)
  );
  setClient(gqlClient);

  let isOnline = false;

  const urlParams = new URLSearchParams(window.location.search);
  const output_id = urlParams.get('output');
  const restream_id = urlParams.get('id');

  const mix = subscribe(Mix, {
    errorPolicy: 'all',
    variables: {
      outputId: output_id,
      restreamId: restream_id,
    },
  });

  $: error = $mix && $mix.error;
  $: isLoading = !isOnline || $mix.loading;
  $: canRenderMainComponent = isOnline && $mix.data;
  $: output = $mix.data && $mix.data.output;
</script>

<template>
  <Shell {canRenderMainComponent} {isLoading} {error}>
    <div slot="main">
      <section class="uk-section uk-section-muted single-output">
        <Output {restream_id} value={output} {mutations} />
      </section>
      {#if isYoutubeVideo(output.previewUrl)}
        <section class="uk-section uk-section-muted video-player">
          <YoutubePlayer {restream_id} preview_url={output.previewUrl} />
        </section>
      {/if}
    </div>
  </Shell>
</template>

<style lang="stylus">
  .single-output
    margin-top: 20px
    padding: 10px 20px 20px 20px
    max-width: 960px

    :global(.volume input)
      width: 90% !important

  .video-player
      @extend .single-output
      max-height: 800px
      min-height: 150px
</style>
