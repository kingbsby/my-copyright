import React, { useEffect, useState } from 'react'
import {
  Form,
  Input,
  Button,
  Image,
  Divider,
  message,
} from 'antd'
import {
  keyStores,
  connect,
  WalletConnection,
  Contract,
} from 'near-api-js'

import Big from 'big.js'

import "./App.css"
import 'antd/dist/antd.css'

const ContractName = 'dev-1646049663373-27224220280109'

const TGas = Big(10).pow(12)
const BoatOfGas = Big(200).mul(TGas)

let _keyStore,
  _nearConfig,
  _near,
  _accountId,
  _account,
  _contract,
  _walletConnection



export default () => {

  const [imgList, setImgList] = useState([])
  const [connected, setConnected] = useState(false)
  const [signedIn, setSignedIn] = useState(false)
  const [accountId, setAccountId] = useState(null)
  const [submitLoding, setSubmitLoding] = useState(false)
  const [buyLoding, setBuyLoding] = useState(false)

  const [form] = Form.useForm()

  useEffect(() => {
    _initNear()
  }, [])

  /**
   * 初始化near
   */
  const _initNear = async () => {

    const nearConfig = {
      networkId: 'default',
      nodeUrl: 'https://rpc.testnet.near.org',
      contractName: ContractName,
      walletUrl: 'https://wallet.testnet.near.org',
    }

    const keyStore = new keyStores.BrowserLocalStorageKeyStore()

    const near = await connect(Object.assign({ deps: { keyStore } }, nearConfig))

    _keyStore = keyStore
    _nearConfig = nearConfig
    _near = near

    _walletConnection = new WalletConnection(near, ContractName)

    _accountId = _walletConnection.getAccountId()

    _account = _walletConnection.account()

    _contract = new Contract(_account, ContractName, {
      viewMethods: ['get_list'],
      changeMethods: ['add_picture', 'buy'],
    })

    setConnected(true)
    setSignedIn(!!_accountId)
    setAccountId(_accountId)

    if (_accountId) {
      await getList()
    }
  }

  /**
   * 获取列表数据
   */
  const getList = async () => {
    const list = await _contract.get_list()
    setImgList(list)
  }

  /**
   * 登录
   */
  const requestSignIn = async () => {
    const appTitle = 'Digital War'
    await _walletConnection.requestSignIn(
      ContractName,
      appTitle
    )
  }

  /**
   * 登出
   */
  const logOut = async () => {
    _walletConnection.signOut()
    _accountId = null

    setSignedIn(false)
    setAccountId(null)

  }

  /**
   * 上传作品
   */
  const onFinish = async ({ opus_name, opus_url, opus_hash, price }) => {
    if (submitLoding) return
    setSubmitLoding(true)

    const pic = await _contract.add_picture({
      opus_name,
      opus_url,
      opus_hash,
      price: parseInt(price)
    })

    setSubmitLoding(false)

    if (pic === 'success') {
      message.success('图片添加成功!')
      form.resetFields()
      getList()
    }
  }

  /**
   * 购买
   * @param {*} img 
   * @returns 
   */
  const buy = (img) => {

    if (buyLoding) return

    setBuyLoding(true)

    _contract.buy({ pic_hash: img.opus_hash }, BoatOfGas.toFixed(0), Big(img.price).mul(1).toFixed(0))

  }

  const signedInDom = (
    <>
      <div className="float-right">
        <button
          className="btn btn-outline-secondary"
          onClick={logOut}>Log out</button>
      </div>
      <h4>Hello, <span className="font-weight-bold">{accountId}</span>!</h4>
      <Divider dashed />
      <Form wrapperCol={{ span: 14 }} form={form} onFinish={onFinish}>
        <Form.Item
          label="图片名称"
          name="opus_name"
          rules={[{ required: true, message: '请输入图片名称' }]}>
          <Input placeholder="请输入图片名称" />
        </Form.Item>
        <Form.Item
          label="图片路径"
          name="opus_url"
          rules={[{ required: true, message: '请输入图片路径' }]}>
          <Input placeholder="请输入图片路径" />
        </Form.Item>
        <Form.Item
          label="图片hash"
          name="opus_hash"
          rules={[{ required: true, message: '请输入图片hash' }]}>
          <Input placeholder="请输入图片hash" />
        </Form.Item>
        <Form.Item
          label="图片价格"
          name="price"
          rules={[{ required: true, message: '请输入图片价格' }]}>
          <Input placeholder="请输入图片价格" />
        </Form.Item>
        <Form.Item >
          <Button
            type="primary"
            htmlType="submit"
            loading={submitLoding}
          >Submit</Button>
        </Form.Item>
      </Form>
      <Divider dashed />
      <ul className='img_list'>
        {
          imgList.map((img, i) => {
            return (
              <li key={i}>
                <Image
                  width={200}
                  height={200}
                  fallback="data:image/png;base64,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"
                  src={img.opus_url}
                />
                <p className='name'>{img.opus_name}</p>
                <p className='owner'>{img.owner}</p>
                <p className='price'>{img.price}</p>
                {
                  img.owner === accountId
                    ? <Button disabled>购买</Button>
                    : <Button loading={buyLoding} onClick={() => buy(img)}>购买</Button>
                }
              </li>
            )
          })
        }
      </ul>
    </>
  )

  const content = !connected
    ? (
      <div>
        Connecting...
        <span className="spinner-grow spinner-grow-sm" role="status" aria-hidden="true"></span>
      </div>
    )
    : (
      signedIn
        ? signedInDom
        : (
          <div style={{ marginBottom: "10px" }}>
            <button
              className="btn btn-primary"
              onClick={requestSignIn}>Log in with NEAR Wallet</button>
          </div>
        )
    )

  return (
    <div className="px-5">
      <h1>COPYRIGHT</h1>
      {content}
    </div>
  )
}