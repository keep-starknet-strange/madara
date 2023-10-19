contract UnsafeProxy {

    address delegate;

    constructor(address _delegate) {
        delegate = _delegate;
    }

    fallback() external payable {
        address target = delegate;
        bytes memory data = msg.data;
        assembly {
            let result := delegatecall(gas(), target, add(data,0x20), mload(data), 0, 0)
            let size := returndatasize()
            let ptr := mload(0x40)
            returndatacopy(ptr,0,size)
            switch result
            case 0 {revert(ptr,size)}
            default {return(ptr,size)}
        }
    }
}
