<script lang="js">
  import { InMemoryCache } from '@apollo/client/cache';
  import { ApolloClient } from '@apollo/client/core';
  import { WebSocketLink } from '@apollo/client/link/ws';
  import { SubscriptionClient } from 'subscriptions-transport-ws';
  import { onDestroy } from 'svelte';
  import { get } from 'svelte/store';
  import { setClient, subscribe } from 'svelte-apollo';
  import Router, { replace } from 'svelte-spa-router';
  import { wrap } from 'svelte-spa-router/wrap';

  import { Info, State } from './api/graphql/client.graphql';
  import { showError } from './util';

  export let mainComponent;
  export let toolbarComponent;
  export let apiUrl;

  import UIkit from 'uikit';
  import Icons from 'uikit/dist/js/uikit-icons';

  import * as PageOutput from './pages/Output.svelte';

  UIkit.use(Icons);

  let isOnline = true;

  const protocol = window.location.protocol === 'https:' ? 'wss' : 'ws';
  const wsClient = new SubscriptionClient(
    `${protocol}://${window.location.hostname}${apiUrl}`,
    { reconnect: true }
  );
  wsClient.onConnected(() => {
    isOnline = true;
  });
  wsClient.onReconnected(() => {
    isOnline = true;
  });
  wsClient.onDisconnected(() => (isOnline = false));
  const gqlClient = new ApolloClient({
    link: new WebSocketLink(wsClient),
    cache: new InMemoryCache(),
  });
  setClient(gqlClient);

  const info = subscribe(Info, { errorPolicy: 'all' });
  const state = subscribe(State, { errorPolicy: 'all' });

  let currentHash = undefined;
  onDestroy(
    info.subscribe((i) => {
      if (i.data) {
        const newHash = i.data.info.passwordHash;
        if (currentHash === undefined) {
          currentHash = newHash;
        } else if (!!newHash && newHash !== currentHash) {
          window.location.reload();
        }

        const title = i.data.info.title;
        document.title = title || 'Ephyr re-streamer';
      }
    })
  );

  const routes = {
    '/id/:restream_id/output/:output_id': wrap({
      component: PageOutput,
      props: {
        state,
      },
      userData: {
        state,
      },
      conditions: [
        (detail) => {
          const st = get(detail.userData.state);
          const p = detail.location.split('/');
          return (
            !!st.data &&
            st.data.allRestreams.some(
              (r) => r.id === p[2] && r.outputs.some((o) => o.id === p[4])
            )
          );
        },
      ],
    }),
    '*': wrap({
      component: mainComponent,
      props: {
        info,
        state,
      },
    }),
  };
</script>

<template>
  <div class="page uk-flex uk-flex-column">
    <header class="uk-container">
      <div class="uk-grid uk-grid-small" uk-grid>
        <a
          href="https://allatraunites.com"
          target="_blank"
          class="logo uk-flex"
          title="Join us on allatraunites.com"
        >
          <img src="logo.jpg" alt="Logo" />
          <h3>Creative Society</h3>
          <small>Ephyr re-streamer {process.env.VERSION}</small>
        </a>
        <div class="uk-margin-auto-left">
          {#if isOnline && $info.data}
            <svelte:component
              this={toolbarComponent}
              info={$info}
              state={$state}
              {isOnline}
              {gqlClient}
            />
          {:else if $info.error}
            {showError($info.error.message) || ''}
          {/if}
        </div>
      </div>
    </header>

    <main class="uk-container uk-flex-1">
      {#if !isOnline || $state.loading}
        <div class="uk-alert uk-alert-warning loading">Loading...</div>
      {:else if isOnline && $state.data && $info.data}
        <Router {routes} on:conditionsFailed={() => replace('/')} />
      {/if}
      {#if $state.error}
        {showError($state.error.message) || ''}
      {/if}
    </main>

    <footer class="uk-container">
      Developed for people with ❤ by
      <a href="https://github.com/ALLATRA-IT" target="_blank">AllatRa IT</a>
    </footer>
  </div>
</template>

<style lang="stylus" global>
  @require "../node_modules/uikit/dist/css/uikit.min.css"
  :root {
    --primary-text-color: #777;
  }

  .page
    min-height: 100vh;

  h2, h3
    color: var(--primary-text-color)

  .uk-container
    padding-left: 34px !important
    padding-right: @padding-left
    max-width: auto !important
    width: calc(100% - 68px)
    min-width: 320px

  header
    padding: 10px

    .logo
      outline: none
      position: relative
      white-space: nowrap
      &:hover
        text-decoration: none

      img
        width: 44px
        height: @width

      h3
        margin: 4px 4px 4px 8px
        max-width: 50%

      small
        position: absolute
        font-size: 12px
        bottom: -6px
        left: 83px
        color: #999

  main
    > .loading
      text-align: center

  .uk-button-primary
    background-color: #08c
    &:not([disabled]):hover
      background-color: #046

  footer
    padding-top: 10px
    padding-bottom: 3px
    font-size: 12px

  .uk-notification-message
    pointer-events: none
    font-size: 1rem
    overflow-wrap: anywhere
    & > div
      padding-right: 14px

    .uk-notification-close
      display: inherit
      pointer-events: all

    .uk-icon-link
      pointer-events: all

  .overflow-wrap
    overflow-wrap: anywhere;
    white-space: normal;

</style>
