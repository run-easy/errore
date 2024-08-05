# RUN Error Library
Building a robust program or application hinges on a well-structured exception handling system. 
To create such a system, adhering to the following principles is crucial:

- Should an exception arise during the execution of a function or method, it must be handled in one of the following ways:
    - If an exception is thrown without impacting the function's execution, it should be ignored.
    - If an exception is generated affecting the function's execution but can be managed within the function, it should be masked.
    - If an exception arises influencing the function's execution and cannot be managed internally, it should be propagated to the bottom of the call stack.

- Any function or method should conform to the following guidelines:
    - The return value of a function must be Result<T,Error> if any of the exceptions generated during its execution cannot be handled internally.
    - Any exceptions generated during the execution of the function can be handled internally, so the return value of the function is of type T.
    - For any call to a function or method that is panic or unsafe marked, it must be shown that the call does not panic, or it must be called with a branch and return Result.

According to the above principles, the design of error types must satisfy the following conditions:
1. The error type structure must be compact and memory small enough to propagate easily.
2. The error type should contain enough information to help the program determine the source and cause and develop a strategy for handling it.
3. Error types cannot be copied, only moved.
4. Error types that are not explicitly handled will panic automatically.
5. Error types allow users to extend them.