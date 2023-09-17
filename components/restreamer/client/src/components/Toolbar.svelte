<script lang="js">
  import Fa from 'svelte-fa';
  import {
    faPlus,
    faShareSquare,
    faCog,
    faLock,
    faLockOpen,
  } from '@fortawesome/free-solid-svg-icons';

  import { exportModal } from '../stores';

  import RestreamModal from '../modals/RestreamModal.svelte';
  import PasswordModal from '../modals/PasswordModal.svelte';
  import ExportModal from '../modals/ExportModal.svelte';

  import SettingsModal from '../modals/SettingsModal.svelte';

  import cloneDeep from 'lodash/cloneDeep';
  import { ExportAllRestreams } from '../../api/client.graphql';
  import { showError } from '../utils/util';

  export let info;
  export let state;
  export let isOnline;
  export let gqlClient;

  async function openExportModal() {
    let resp;
    try {
      resp = await gqlClient.query({
        query: ExportAllRestreams,
        fetchPolicy: 'no-cache',
      });
    } catch (e) {
      showError(e.message);
      return;
    }

    if (!!resp.data) {
      exportModal.open(
        null,
        resp.data.export
          ? JSON.stringify(JSON.parse(resp.data.export), null, 2)
          : ''
      );
    }
  }

  let openPasswordModal = false;
  let openSettingsModal = false;
  let openRestreamModal = false;
</script>

<template>
  <a
    href="/"
    class="set-settings"
    title="Change settings"
    on:click|preventDefault={() => (openSettingsModal = true)}
  >
    <Fa icon={faCog} />
  </a>
  {#if openSettingsModal}
    <SettingsModal
      info={cloneDeep($info.data.info)}
      bind:visible={openSettingsModal}
    />
  {/if}
  {#key $info.data.info.passwordHash}
    <a
      href="/"
      class="set-password"
      title="{!$info.data.info.passwordHash ? 'Set' : 'Change'} password"
      on:click|preventDefault={() => (openPasswordModal = true)}
    >
      {#if $info.data.info.passwordHash}
        <Fa icon={faLock} />
      {:else}
        <Fa icon={faLockOpen} />
      {/if}
    </a>
    {#if openPasswordModal}
      <PasswordModal
        password_kind="MAIN"
        current_hash={$info.data.info.passwordHash}
        bind:visible={openPasswordModal}
      />
    {/if}
  {/key}
  <div class="add-input">
    <button
      data-testid="add-input:open-modal-btn"
      class="uk-button uk-button-primary"
      on:click={() => (openRestreamModal = true)}
    >
      <Fa icon={faPlus} />
      <span>Input</span>
    </button>
    {#if openRestreamModal}
      <RestreamModal
        public_host={$info.data.info.publicHost}
        bind:visible={openRestreamModal}
      />
    {/if}

    {#if isOnline && $state.data}
      <ExportModal />
      <a
        class="export-import-all"
        href="/"
        on:click|preventDefault={openExportModal}
        title="Export/Import all"
      >
        <Fa icon={faShareSquare} />
      </a>
    {/if}
  </div>
</template>

<style lang="stylus">
  .set-password, .set-settings
    margin-right: 26px
    font-size: 26px
    color: var(--primary-text-color)
    outline: none

    &:hover
      text-decoration: none
      color: #444

  .add-input
    position: relative
    display: inline-block
    vertical-align: top

  .export-import-all
    position: absolute
    top: 6px
    right: -24px
    opacity: 0
    transition: opacity .3s ease
    color: var(--primary-text-color)
    outline: none

    &:hover
      text-decoration: none
      color: #444
      opacity: 1

  &:hover
    .export-import-all
      opacity: 1

</style>
